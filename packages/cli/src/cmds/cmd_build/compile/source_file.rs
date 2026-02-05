use std::{ffi::OsStr, path::{Path, PathBuf}};

use cu::{Context, str::OsStrExtension};

use crate::{cmds::cmd_build::{compile::compile_command::CompileCommand, compile_db::CompileRecord, config::Flags}, env::environment};

// Specifies source language (rust is managed separately)
#[derive(PartialEq, Eq, Debug)]
pub enum Lang {
    C,
    Cpp,
    S,
    None
}
impl Lang {
    pub fn get_compiler_path(&self) -> &Path {
        match self {
            Lang::C => environment().cc_path(),
            Lang::Cpp =>  environment().cxx_path(),
            Lang::S =>  environment().cc_path(),
            Lang::None => panic!("Error in compiler_path: No compiler defined for {:?}", self),
        }
    }

    pub fn get_flags<'a>(&self, flags: &'a Flags) -> &'a Vec<String> {
        match self {
            Lang::C => &flags.cflags,
            Lang::Cpp =>  &flags.cxxflags,
            Lang::S =>  &flags.cflags,
            Lang::None => panic!("Error in get_flags: No flags specified for {:?}", self),
        }
    }
}


// Get every source file in the given directory, recursivly
// Skips entrys with an unknown extension
// Warns if an entry cannot be read for some reason
pub fn discover_source_files(dir: &Path) -> cu::Result<Vec<SourceFile>> {
    let mut sources = Vec::new();
    let mut walk = cu::fs::walk(dir)?;
    while let Some(walk_result) = walk.next() {
        let entry = walk_result.context("failed to walk source directory")?;
        let path = entry.path();
        let lang = match path.extension().and_then(OsStr::to_str).unwrap_or_default() {
            "c" => Some(Lang::C),
            "cpp" | "c++" | "cc" | "cxx" => Some(Lang::Cpp),
            "s" | "asm" => Some(Lang::S),
            _ => None,
        };
        if let Some(_lang) = lang {
            let source = SourceFile::new(path).context("failed to create source object")?;
            sources.push(source);
        } else {
            cu::debug!(
                "Unrecognized extension: {}, skipping",
                path.to_str().unwrap_or("illegible filename")
            );
        }
    }
    Ok(sources)
}

// A source file and its corresponding artifacts
#[derive(Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub pathhash: usize,
    pub basename: String,
}


impl SourceFile {
    pub fn get_lang(&self) -> Lang {
        match self.path.extension().and_then(OsStr::to_str).unwrap_or_default() {
            "c" => Lang::C,
            "cpp" | "c++" | "cc" | "cxx" => Lang::Cpp,
            "s" | "asm" => Lang::S,
            _ => Lang::None,
        }
    }

    pub fn get_o_path(&self, output_path: &Path) -> PathBuf {
        output_path.join(format!("{}-{}.o", self.basename, self.pathhash))
    }
    pub fn get_d_path(&self, output_path: &Path) -> PathBuf {
        output_path.join(format!("{}-{}.d", self.basename, self.pathhash))
    }
    
    pub fn new(path: PathBuf) -> cu::Result<Self> {
        let basename = cu::str::PathExtension::file_name_str(&path)
            .context("path is not utf-8")?
            .to_owned();
        let pathhash = fxhash::hash(&path);
        Ok(Self {
            path,
            pathhash,
            basename,
        })
    }

    pub fn build_compile_command(&self, output_path: &Path, flags: &Flags, includes: &[String]) -> CompileCommand {
        let lang = self.get_lang();
        let compiler_path = lang.get_compiler_path();
        let flags = lang.get_flags(flags);
        let o_path = self.get_o_path(output_path);
        let d_path = self.get_d_path(output_path);

        CompileCommand::new(
            compiler_path, &self.path, &o_path, &d_path, flags, includes,
        )
    }

    pub fn need_recompile(
        &self,
        old_record: Option<&CompileRecord>,
        output_path: &Path,
        command: &CompileCommand,
    ) -> cu::Result<bool> {
        if old_record.is_none() {
            // Check no prior compilation recorded
            return Ok(true);
        }
        let old_record = old_record.unwrap();

        if command.args != old_record.args {
            // arguments don't match - recompiling would lead to a newer result
            return Ok(true);
        }

        let o_path = self.get_o_path(output_path);
        let d_path = self.get_d_path(output_path);

        if !o_path.exists() || !d_path.exists() {
            // artifacts don't exist, so we can't re-use them anyway
            return Ok(true);
        }

        if cu::fs::get_mtime(&o_path)? != cu::fs::get_mtime(&self.path)?
        || cu::fs::get_mtime(&d_path)? != cu::fs::get_mtime(&self.path)?
        {
            // Our source code has changed since last compilation
            return Ok(true);
        }

        let d_file_contents = cu::fs::read_string(&d_path)?;
        let depfile = match depfile::parse(&d_file_contents) {
            Ok(depfile) => depfile,

            // Make sure our errors are all cu compatible
            Err(_) => return Err(cu::Error::msg("Failed to parse depfile")),
        };

        for dep in depfile.recurse_deps(o_path.as_utf8()?) {
            if cu::fs::get_mtime(PathBuf::from(dep))? != cu::fs::get_mtime(&self.path)? {
                return Ok(true);
            }
        }

        // if (self.lang == Lang::Cpp && compile_db.cxx_version != env.cxx_version)
        //     || compile_db.cc_version != env.cc_version
        // {
        //     return Ok(true);
        // }

        // No need to recompile!
        Ok(false)
    }

}

pub fn get_pathhash(path: &PathBuf) -> usize {
    fxhash::hash(&path)
}
