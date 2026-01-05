// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

// This modules handles compiling c/c++/asm/rust code
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use cu::pre::*;

use super::{Flags, RustCrate};
use crate::{cmds::cmd_build::config::{Build, Module}, env::environment};

#[derive(Serialize, Deserialize, Default)]
pub struct CompileDB {
    // key: name of source
    commands: BTreeMap<String, CompileRecord>,
    cc_version: String,
    cxx_version: String,
}

impl CompileDB {
    fn new() -> Self {
        Self::default()
    }

    // Creates a new compile record and adds it to the db
    fn update(&mut self, command: CompileCommand) -> cu::Result<()> {
        todo!()
    }
}


#[derive(Serialize, Deserialize, Default)]
pub struct MyCompileDB {
    commands: Vec<CompileCommand>,
    cc_version: String,
    cxx_version: String,
    pub ld_command: String,
}

impl MyCompileDB {
    fn new() -> Self {
        Self::default()
    }

    // Creates a new compile record and adds it to the db
    pub fn update(&mut self, command: CompileCommand) -> cu::Result<()> {
        self.commands.push(command);
        Ok(())
    }

    fn set_linker_command(&mut self, cmd: String) {
        self.ld_command = cmd;
    }

    pub fn save(&self) -> cu::Result<()> {
        let path = PathBuf::new().join("compilecommands.txt");
        let content = self.commands.iter()
            .map(|c| format!("{} {}", 
                c.compiler.as_os_str().to_str().unwrap(), 
                c.args.join(" ")))
            
            .collect::<Vec<String>>()
            .join("\n\n");
        cu::fs::write(path, content)
    }
}

#[derive(Serialize, Deserialize)]
struct CompileRecord {
    command: CompileCommand,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CompileCommand {
    compiler: PathBuf,
    source: PathBuf,
    args: Vec<String>,
}

impl CompileCommand {
    pub fn new_ld_command(ld_path: &Path, args: &Vec<String>) -> Self {
        Self { compiler: ld_path.to_path_buf(), source: PathBuf::new(), args: args.to_vec() }
    }

    fn new(
        compiler_path: &Path,
        src_file: &Path,
        out_file: &Path,
        dep_file: &Path,
        flags: &Vec<String>,
        build_config: &Build,
    ) -> cu::Result<Self> {
        let mut argv = flags.clone();
        argv.push("-MMD".to_owned());
        argv.push("-MP".to_owned());
        argv.push("-MF".to_owned());
        argv.push(dep_file.as_utf8()?.to_string());

        let mut includes = build_config.includes.clone();
        includes.extend(vec!["/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/15.1.0".to_owned(),
                            "/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/15.1.0/aarch64-none-elf".to_owned(),
                            "/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/15.1.0/backward".to_owned(),
                            "/opt/devkitpro/devkitA64/lib/gcc/aarch64-none-elf/15.1.0/include".to_owned(),
                            "/opt/devkitpro/devkitA64/lib/gcc/aarch64-none-elf/15.1.0/include-fixed".to_owned(),
                            "/opt/devkitpro/devkitA64/aarch64-none-elf/include".to_owned()
        ]);
        let includes = includes.iter()
            .filter_map(|i| {
                let path = PathBuf::from(i);
                cu::info!("{:?}", path);
                path.canonicalize().inspect_err(|e| {cu::error!("cant find include {:?}", e)}).ok()
            })
            .map(|i| format!("-I{}", i.as_os_str().to_str().unwrap()))
            .collect::<Vec<String>>();
        argv.extend(includes);

        argv.push(String::from("-c"));

        argv.push(format!("-o{}", out_file.as_utf8()?));
        
        argv.push(src_file.as_utf8()?.to_string());

        cu::info!("Compiler command: \n{} {} {}", &compiler_path.display(), &src_file.display(), &argv.join(" "));

        Ok(Self {
            compiler: compiler_path.to_path_buf(),
            source: src_file.to_path_buf(),
            args: argv,
        })
    }

    fn execute(&self) -> cu::Result<()> {
        cu::info!("Executing CompileCommand: \n{} {}", &self.compiler.display(), &self.args.join(" "));
        let child = cu::CommandBuilder::new(&self.compiler.as_os_str())
            .args(self.args.clone())
            .stdoe(cu::pio::inherit()) // todo: log to file
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        Ok(())
    }

    // We need two different ways of serializing this data since it will
    // need to be writen to the compiledb cache and the compile_commands.json
    // and the format will be different for each
    fn to_clangd_json(&self) -> String {
        // TODO: Implement
        todo!()
    }
    
}

// A source file and its corresponding artifacts
pub struct SourceFile {
    lang: Lang,
    path: PathBuf,
    pathhash: usize,
    basename: String,
}

impl SourceFile {
    pub fn new(lang: Lang, path: PathBuf) -> cu::Result<Self> {
        let basename = cu::PathExtension::file_name_str(&path)
            .context("path is not utf-8")?
            .to_owned();
        let pathhash = fxhash::hash(&path);
        Ok(Self {
            lang,
            path,
            pathhash,
            basename,
        })
    }

    pub fn compile(
        &self,
        flags: &Flags,
        build: &Build,
        compile_db: &mut CompileDB,
        other_compile_db: &mut MyCompileDB,
        module_path: &Path,
    ) -> cu::Result<bool> {
        let output_path = module_path
            .join("o");
        if !output_path.exists() {
            cu::fs::make_dir(&output_path).unwrap();
            cu::info!("Output path {:?} exists={}", &output_path, &output_path.exists());
        }
        let o_path = output_path
            .join(format!("{}-{}.o", self.basename, self.pathhash));
        let d_path = output_path
            .join(format!("{}-{}.d", self.basename, self.pathhash));

        let (comp_path, comp_flags) = match self.lang {
            Lang::C => (environment().cc_path(), &flags.cflags),
            Lang::Cpp => (environment().cxx_path(), &flags.cxxflags),
            Lang::S => (environment().cc_path(), &flags.sflags),
        };

        let comp_command =
            CompileCommand::new(comp_path, &self.path, &o_path, &d_path, &comp_flags, build)?;

        other_compile_db.update(comp_command.clone());

        if self.need_recompile(compile_db, &o_path, &d_path, &comp_command)? {
            // Ensure source and artifacts have the same timestamp
            let src_time = cu::fs::get_mtime(&self.path)?.unwrap();

            // Compile and update record
            comp_command.execute()?;

            cu::fs::set_mtime(o_path, src_time)?;
            cu::fs::set_mtime(d_path, src_time)?;
            Ok(true)
            // compile_db.update(comp_command)?;            
        } else {
            Ok(false)
        }
    }

    fn need_recompile(
        &self,
        compile_db: &CompileDB,
        o_path: &Path,
        d_path: &Path,
        command: &CompileCommand,
    ) -> cu::Result<bool> {
        // Check if record exists
        let comp_record = match compile_db.commands.get(&self.basename) {
            Some(record) => record,
            None => return Ok(true),
        };

        // Check if artifacts exist
        if !o_path.exists() || !d_path.exists() {
            return Ok(true);
        }

        // Check if artifacts are up to date
        if cu::fs::get_mtime(o_path)? != cu::fs::get_mtime(&self.path)?
            || cu::fs::get_mtime(d_path)? != cu::fs::get_mtime(&self.path)?
        {
            return Ok(true);
        }

        let d_file_contents = cu::fs::read_string(d_path)?;
        let depfile = match depfile::parse(&&d_file_contents) {
            Ok(depfile) => depfile,

            // Make sure our errors are all cu compatible
            Err(_) => return Err(cu::Error::msg("Failed to parse depfile")),
        };

        for dep in depfile.recurse_deps(o_path.as_utf8()?) {
            if cu::fs::get_mtime(PathBuf::from(dep))? != cu::fs::get_mtime(&self.path)? {
                return Ok(true);
            }
        }

        if command != &comp_record.command {
            return Ok(true);
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

// Specifies source language (rust is managed separately)
#[derive(PartialEq, Eq)]
pub enum Lang {
    C,
    Cpp,
    S,
}

// Builds the give rust crate and places the binary in the target as specified in the rust manifest
pub fn compile_rust(rust_crate: RustCrate) -> cu::Result<()> {
    // TODO: Implement
    Ok(todo!())
}


fn get_obj_files_in(target: &PathBuf) -> Vec<PathBuf> {
    let paths = cu::fs::read_dir(target).unwrap();
    let mut result: Vec<PathBuf> = vec![];

    for path in paths {
        if let Ok(path) = path
            && let Ok(ft) = path.file_type()
        {
            if ft.is_dir() {
                result.extend(get_obj_files_in(&path.path()));
            } else if ft.is_file() && is_file_obj(&path) {
                result.push(path.path());
            }
        }
    }
    result
}

fn is_file_obj(path: &cu::fs::DirEntry) -> bool {
    let name = path.file_name();
    let name = name.to_str();
    if let Some(name) = name {
        name.ends_with(".o")
    } else {
        false
    }
}

pub fn relink(
    module_path: &PathBuf,
    compdb: &mut CompileDB,
    my_compdb: &mut MyCompileDB,
    libraries: &Vec<String>,
    module: &Module,
    flags: &Flags,
) -> cu::Result<bool> {
    // Returns: whether linking was performed

    let elf_name = format!("{}.elf",module.name);
    let elf_path = module_path.join(elf_name);
    
    let obj_files: Vec<String> = get_obj_files_in(module_path)
        .iter().map(|o| o.display().to_string()).collect(); // scan target folder (BuildConfig)
    let env = environment();

    let output_arg = ["-o".to_string(), elf_path.display().to_string()];
    let mut args = flags.ldflags.clone();
    args.extend(libraries.iter().map(|lib| lib.to_owned()));
    args.extend(obj_files);
    args.extend(output_arg);
    let cxx = env.cxx_path();
    let link_cmd = CompileCommand::new_ld_command(cxx, &args);
    // todo: implement proper equality check (same args in the same order?)
    if let Some(old_link_cmd) = &compdb.commands.last_entry() && old_link_cmd.get().command.eq(&link_cmd) { 
        return Ok(false); // no relinking needed
    }
    my_compdb.update(link_cmd).unwrap();

    cu::info!("{:?}", &args);
    let command = cxx.command()
        .args(
            args,
        )
        .stdin_null()
        .stdoe(cu::pio::spinner("linking").info())
        .spawn()?
        .0;
    
    command.wait_nz()?;
    cu::info!("Linker finished!");
    Ok(true)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_obj_files() {
        let mut path = PathBuf::from("test/test_get_obj_files");
        let result = get_obj_files_in(&path);
        assert!(
            result.contains(&PathBuf::from("test/test_get_obj_files/file1.o")),
            "{:#?}",
            result
        );
        assert!(
            result.contains(&PathBuf::from("test/test_get_obj_files/nested/file2.o")),
            "{:#?}",
            result
        );
    }

    #[test]
    fn test_get_obj_files_empty() {
        let mut path = PathBuf::from("test/test_get_obj_files/empty");
        let result = get_obj_files_in(&path);
        assert_eq!(result.len(), 0, "{:#?}", result);
    }
}
