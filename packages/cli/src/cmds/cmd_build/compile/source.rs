use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use cu::pre::*;

use super::compile_db::CompileRecord;
use crate::{cmds::cmd_build::config::Flags, env::environment};

// A source file and its corresponding artifacts
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub pathhash: usize,
    basename: String,
    lang: Lang,
}

impl SourceFile {
    /// Returns a source file object, or None, if the given file is not a valid source
    /// i.e. the name cannot be parsed or the file extension is invalid
    pub fn new(path: PathBuf) -> Option<Self> {
        let basename = path.file_name();
        let basename = match basename {
            Some(name) => name.display().to_string(),
            None => return None,
        };

        let lang = match path.extension().and_then(OsStr::to_str).unwrap_or_default() {
            "c" => Lang::C,
            "cpp" | "c++" | "cc" | "cxx" => Lang::Cpp,
            "s" | "asm" => Lang::S,
            _ => return None,
        };

        Some(Self {
            path: path.to_owned(),
            pathhash: fxhash::hash(&path),
            basename,
            lang,
        })
    }

    pub fn get_o_path(&self, output_path: &Path) -> PathBuf {
        output_path.join(format!("{}-{}.o", self.basename, self.pathhash))
    }
    pub fn get_d_path(&self, output_path: &Path) -> PathBuf {
        output_path.join(format!("{}-{}.d", self.basename, self.pathhash))
    }

    /// Compiles all sources in the context
    /// Return values:
    /// 0 : bool - true if anything actually compiled, false if compilation was skipped
    /// 1 : CompileRecord - record of the most recent compilation
    pub async fn compile(
        self,
        flags: Arc<Flags>,
        output_path: Arc<PathBuf>,
        record: Option<CompileRecord>,
    ) -> cu::Result<(bool, CompileRecord)> {
        let compiler = self.lang.get_compiler_path();
        let mut args = self.lang.get_flags(&flags).clone();

        let o_path = self.get_o_path(&output_path);
        let d_path = self.get_d_path(&output_path);

        // Add arguments to generate depfile
        args.push("-MMD".to_string());
        args.push("-MP".to_string());
        args.push("-MF".to_string());
        args.push(d_path.display().to_string());

        args.push("-c".to_string());
        args.push(format!("-o{}", o_path.display()));
        args.push(self.path.display().to_string());

        if self.up_to_date(&record, &output_path, &args)? {
            cu::debug!("{} up to date", o_path.display());
            return Ok((false, record.unwrap()));
        }

        let start_time = cu::fs::Time::now();

        cu::debug!(
            "compiling: {} -> {}\n|\n{} {}\n\n",
            self.path.display(),
            o_path.display(),
            compiler.display(),
            args.join(" ")
        );
        compiler
            .command()
            .stdout(cu::lv::T)
            .stderr(cu::lv::E)
            .stdin_null()
            .args(&args)
            .co_spawn()
            .await?
            .co_wait_nz()
            .await?;

        cu::fs::set_mtime(&self.path, start_time)?;
        cu::fs::set_mtime(o_path, start_time)?;
        if d_path.exists() {
            cu::fs::set_mtime(d_path, start_time)?;
        }

        Ok((
            true,
            CompileRecord {
                source_path: self.path,
                source_hash: self.pathhash,
                compiler: compiler.to_owned(),
                args,
            },
        ))
    }

    fn up_to_date(
        &self,
        record: &Option<CompileRecord>,
        output_path: &Path,
        args: &[String],
    ) -> cu::Result<bool> {
        // Chech that record exists
        let record = match record {
            Some(rec) => rec,
            None => {
                return Ok(false);
            }
        };

        // Check that arguments have not changed
        if args != record.args {
            return Ok(false);
        }

        let o_path = self.get_o_path(output_path);
        let d_path = self.get_d_path(output_path);

        // Check that artifacts exist
        if !o_path.exists() || (!d_path.exists() && self.lang != Lang::S) {
            return Ok(false);
        }

        // Check that source time = o time
        if cu::fs::get_mtime(&o_path)? != cu::fs::get_mtime(&self.path)? {
            return Ok(false);
        }

        // Assembly files don't need dependency checks
        if self.lang == Lang::S {
            return Ok(true);
        }

        // Check that source time = d time
        if cu::fs::get_mtime(&d_path)? != cu::fs::get_mtime(&self.path)? {
            return Ok(false);
        }

        let d_file_contents = cu::fs::read_string(&d_path)?;
        let depfile = match depfile::parse(&d_file_contents) {
            Ok(depfile) => depfile,
            Err(pos) => {
                return Err(cu::Error::msg(format!(
                    "Error when parsing depfile {} at position {pos}",
                    d_path.display()
                )));
            }
        };

        // Check that all dependencies are up to date
        for dep in depfile.recurse_deps(o_path.as_utf8()?) {
            if cu::fs::get_mtime(PathBuf::from(dep))?.unwrap()
                > cu::fs::get_mtime(&self.path)?.unwrap()
            {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Lang {
    C,
    Cpp,
    S,
}

impl Lang {
    fn get_compiler_path(&self) -> &Path {
        match self {
            Lang::C => environment().cc_path(),
            Lang::Cpp => environment().cxx_path(),
            Lang::S => environment().cc_path(),
        }
    }

    fn get_flags<'a>(&self, flags: &'a Flags) -> &'a Vec<String> {
        match self {
            Lang::C => &flags.cflags,
            Lang::Cpp => &flags.cxxflags,
            Lang::S => &flags.cflags,
        }
    }
}

pub struct SourceIterator {
    walks: Vec<cu::fs::Walk>,
    num_walks: usize,
    idx: usize,
}

impl Iterator for SourceIterator {
    type Item = SourceFile;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.num_walks {
            if let Some(Ok(entry)) = self.walks[self.idx].next() {
                if let Some(source) = SourceFile::new(entry.path()) {
                    return Some(source);
                }
            }
            self.idx += 1;
        }
        None
    }
}

pub fn scan(dirs: &[PathBuf]) -> SourceIterator {
    let walks = dirs
        .iter()
        .filter_map(|dir| cu::fs::walk(dir).ok())
        .collect::<Vec<_>>();
    let num_walks = walks.len();

    let src_iter = SourceIterator {
        walks,
        num_walks,
        idx: 0,
    };

    src_iter
}
