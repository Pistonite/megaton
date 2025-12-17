// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

// This modules handles compiling c/c++/asm/rust code
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use cu::pre::*;
use fxhash::hash;

use super::{Flags, RustCrate};
use crate::{cmds::cmd_build::config::Build, env::environment};

#[derive(Serialize, Deserialize, Default)]
pub struct CompileDB {
    commands: BTreeMap<String, CompileRecord>,
    cc_version: String,
    cxx_version: String,
    pub ld_command: String,
}

impl CompileDB {
    fn new() -> Self {
        Self::default()
    }

    // Creates a new compile record and adds it to the db
    fn update(&mut self, command: CompileCommand) -> cu::Result<()> {
        todo!()
    }

    fn set_linker_command(&mut self, cmd: String) {
        self.ld_command = cmd;
    }
}

#[derive(Serialize, Deserialize)]
struct CompileRecord {
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
        dep_file: &Path,
        flags: &Vec<String>,
    ) -> cu::Result<Self> {
        let mut argv = flags.clone();
        argv.push(
            src_file
                .as_utf8()?
                .to_string(),
        );
        argv.push(String::from("-c"));
        argv.push(format!(
            "-o{}
            -MMD -MP -MF{}",
            out_file.as_utf8()?,
            dep_file.as_utf8()?,
        ));

        Ok(Self {
            compiler: compiler_path.to_path_buf(),
            source: src_file.to_path_buf(),
            args: argv,
        })
    }

    fn execute(&self) -> cu::Result<()> {
        cu::info!("Compiling: {:#?}", &self.args);
        cu::CommandBuilder::new(&self.compiler.as_os_str())
            .args(self.args.clone())
            .stdoe(cu::pio::inherit()) // todo: log to file
            .stdin_null()
            .spawn()?;
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
        build: &Build,
        compile_db: &mut CompileDB,
        module_path: &Path
    ) -> cu::Result<()> {
        let o_path = module_path.join("o").join(format!("{}-{}.o", self.basename, self.hash));
        let d_path = module_path.join("o").join(format!("{}-{}.d", self.basename, self.hash));

        let (comp_path, comp_flags) = match self.lang {
            Lang::C => (environment().cc_path(), &flags.cflags),
            Lang::Cpp => (environment().cxx_path(), &flags.cxxflags),
            Lang::S => (environment().cc_path(), &flags.sflags),
        };

        let comp_command = CompileCommand::new(comp_path, &self.path, &o_path, &d_path, &comp_flags)?;

        if self.need_recompile(compile_db, &o_path, &d_path, &comp_command)? {
            // Ensure source and artifacts have the same timestamp
            let src_time = cu::fs::get_mtime(&self.path)?.unwrap();

            // Compile and update record
            comp_command.execute()?;

            cu::fs::set_mtime(o_path, src_time)?;
            cu::fs::set_mtime(d_path, src_time)?;

            compile_db.update(comp_command)?;
        }

        Ok(())
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
