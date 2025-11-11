// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};

use cu::pre::*;

use super::{Flags, Lang, RustCrate, SourceFile};
use crate::{cmds::cmd_build::BuildEnvironment, env::environment};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct CompileCommand {
    compiler: PathBuf,
    source: PathBuf,
    args: Vec<String>,
}

impl CompileCommand {
    fn new(compiler_path: &Path, src_file: &Path, out_file: &Path, flags: &Vec<String>) -> Self {
        let mut argv = flags.clone();
        argv.push(src_file.to_string_lossy().into_owned());
        argv.push(String::from("-c"));
        argv.push(format!("-o {}", out_file.to_string_lossy()));

        Self {
            compiler: compiler_path.to_path_buf(),
            source: src_file.to_path_buf(),
            args: argv,
        }
    }

    fn execute(&self) -> cu::Result<()> {
        // TODO: Build and execute a cu::command
        todo!()
    }

    // We need two different ways of serializing this data since it will
    // need to be writen to the compiledb cache and the compile_commands.json
    // and the format will be different for each
    fn to_clangd_json(&self) -> String {
        // TODO: Implement
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
struct CompileRecord {
    src_basename: String,
    command: CompileCommand,
    last_comp_time: SystemTime,
}

#[derive(Serialize, Deserialize)]
pub struct CompileDB {
    commands: Vec<CompileRecord>,
    cc_version: String,
    cxx_version: String,
}

impl CompileDB {
    // Creates a new compile record and adds it to the db
    fn update(&mut self, command: CompileCommand) -> cu::Result<()> {
        todo!()
    }
}

struct DFile {
    dependent: PathBuf,
    dependencies: Vec<PathBuf>,
}

impl DFile {
    // Maybe can do this easier with serde??
    fn load(path: &Path) -> Self {
        todo!()
    }
}

// Compiles the given source file and writes it to `out`
pub fn compile(
    src: &SourceFile,
    flags: &Flags,
    build_env: &BuildEnvironment,
    compile_db: CompileDB,
) -> cu::Result<()> {
    let name = src.path.file_stem().unwrap().to_str().unwrap();
    let hash = todo!(); // Hash the file

    let o_path = PathBuf::from(format!(
        "{}/{}/{}/o/{name}-{hash}.o",
        build_env.target, build_env.profile, build_env.module
    ));
    let d_path = PathBuf::from(format!(
        "{}/{}/{}/o/{name}-{hash}.d",
        build_env.target, build_env.profile, build_env.module
    ));

    let (comp_path, comp_flags) = match src.lang {
        Lang::C => (environment().cc_path(), &flags.cflags),
        Lang::Cpp => (environment().cxx_path(), &flags.cxxflags),
        Lang::S => (environment().cc_path(), &flags.sflags),
    };

    let comp_command = CompileCommand::new(comp_path, &src.path, &o_path, &comp_flags);

    // Check if record exists
    if let Some(comp_record) = compile_db.commands.iter().find(|r| r.src_basename == name) {
        let src_meta = src
            .path
            .metadata()
            .context("Failed to get src file metadata")?;
        let o_meta = o_path
            .metadata()
            .context("Failed to get .o file metadata")?;
        let d_meta = d_path
            .metadata()
            .context("Failed to get .d file metadata")?;

        // Check if src or artifacts have been modified since last compile
        if o_path.exists()
            && o_meta.modified().unwrap() < comp_record.last_comp_time
            && d_path.exists()
            && d_meta.modified().unwrap() < comp_record.last_comp_time
            && src_meta.modified().unwrap() < comp_record.last_comp_time
        {
            // Check if any dependency has been modified
            let d_file = DFile::load(&d_path);
            if d_file.dependencies.iter().all(|dep| {
                let dep_meta = dep.metadata().unwrap();
                dep_meta.modified().unwrap() < comp_record.last_comp_time
            }) {
                if comp_command == comp_record.command {
                    if (compile_db.cc_version == build_env.cc_version)
                        || (src.lang == Lang::Cpp && compile_db.cc_version == build_env.cc_version)
                    {
                        // No need to recompile!
                        return Ok(());
                    }
                }
            }
        }
    }

    // Compile and update record
    comp_command.execute()?;
    compile_db.update(comp_command)?;

    Ok(())
}

// Builds the give rust crate and places the binary in the target as specified in the rust manifest
pub fn compile_rust(rust_crate: RustCrate) -> cu::Result<()> {
    // TODO: Implement
    Ok(todo!())
}
