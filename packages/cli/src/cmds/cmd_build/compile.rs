// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::path::{Path, PathBuf};

use cu::pre::*;
use fxhash::hash;

use super::{Flags, RustCrate};
use crate::{cmds::cmd_build::BuildEnvironment, env::environment};

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

#[derive(Serialize, Deserialize)]
struct CompileRecord {
    src_basename: String,
    command: CompileCommand,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct CompileCommand {
    compiler: PathBuf,
    source: PathBuf,
    args: Vec<String>,
}

impl CompileCommand {
    fn new(
        compiler_path: &Path,
        src_file: &Path,
        out_file: &Path,
        flags: &Vec<String>,
    ) -> cu::Result<Self> {
        let mut argv = flags.clone();
        argv.push(
            cu::PathExtension::as_utf8(src_file)
                .context("failed to parse utf-8")?
                .to_string(),
        );
        argv.push(String::from("-c"));
        argv.push(format!(
            "-o{}",
            cu::PathExtension::as_utf8(out_file).context("failed to parse utf-8")?
        ));

        Ok(Self {
            compiler: compiler_path.to_path_buf(),
            source: src_file.to_path_buf(),
            args: argv,
        })
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

// A source file and its corresponding artifacts
pub struct SourceFile {
    lang: Lang,
    path: PathBuf,
    basename: String,
    hash: usize,
}

impl SourceFile {
    pub fn new(lang: Lang, path: PathBuf) -> cu::Result<Self> {
        let basename = cu::PathExtension::file_name_str(&path)
            .context("path is not utf-8")?
            .to_owned();
        let hash = hash(&cu::fs::read(&path).context("Failed to read source file")?);
        Ok(Self {
            lang,
            path,
            basename,
            hash,
        })
    }

    pub fn compile(
        &self,
        flags: &Flags,
        build_env: &BuildEnvironment,
        compile_db: &mut CompileDB,
    ) -> cu::Result<()> {
        let o_path = PathBuf::from(format!(
            "{}/{}/{}/o/{}-{}.o",
            build_env.target, build_env.profile, build_env.module, self.basename, self.hash
        ));
        let d_path = PathBuf::from(format!(
            "{}/{}/{}/o/{}-{}.d",
            build_env.target, build_env.profile, build_env.module, self.basename, self.hash
        ));

        let (comp_path, comp_flags) = match self.lang {
            Lang::C => (environment().cc_path(), &flags.cflags),
            Lang::Cpp => (environment().cxx_path(), &flags.cxxflags),
            Lang::S => (environment().cc_path(), &flags.sflags),
        };

        let comp_command = CompileCommand::new(comp_path, &self.path, &o_path, &comp_flags)?;

        if self.need_recompile(compile_db, build_env, &o_path, &d_path)? {
            // Compile and update record
            comp_command.execute()?;

            // Ensure source and artifacts have the same timestamp
            let now = cu::fs::Time::now();
            cu::fs::set_mtime(o_path, now)?;
            cu::fs::set_mtime(d_path, now)?;
            cu::fs::set_mtime(&self.path, now)?;

            compile_db.update(comp_command)?;
        }

        Ok(())
    }

    fn need_recompile(
        &self,
        compile_db: &CompileDB,
        build_env: &BuildEnvironment,
        o_path: &Path,
        d_path: &Path,
    ) -> cu::Result<bool> {
        // Check if record exists
        let comp_record = match compile_db
            .commands
            .iter()
            .find(|r| r.src_basename == self.basename)
        {
            Some(record) => record,
            None => return Ok(true),
        };

        // Check if artifacts exist
        if !o_path.exists() || !d_path.exists() {
            return Ok(true);
        }

        // Check if artifacts are up to date
        if cu::fs::get_mtime(o_path)? != cu::fs::get_mtime(self.path)?
            || cu::fs::get_mtime(d_path)? != cu::fs::get_mtime(self.path)?
        {
            return Ok(true);
        }

        let d_file =
            match depfile::parse(&cu::fs::read_string(d_path).context("Failed to read depfile")?) {
                Ok(depfile) => depfile,

                // Make sure our errors are all cu compatible
                Err(byte) => return Err(cu::Error::msg("Failed to parse depfile")),
            };

        if d_file.dependencies.iter().all(|dep| {
            let dep_meta = dep.metadata().unwrap();
            dep_meta.modified().unwrap() < comp_record.last_comp_time
        }) {
            if comp_command == comp_record.command {
                if (compile_db.cc_version == build_env.cc_version)
                    || (src.lang == Lang::Cpp && compile_db.cc_version == build_env.cc_version)
                {
                    return Ok(false);
                }
            }
        }

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
