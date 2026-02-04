
// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

// This module handles scanning the mod/library source

use std::io::Write;
use std::sync::Arc;
use std::{ffi::OsStr, path::PathBuf};
use std::path::Path;

use cu::pre::*;
use regex::Regex;

use crate::cmds::cmd_build::compile::source_file::Lang;
use crate::cmds::cmd_build::config::Flags;
use crate::env::environment;


// Get every source file in the given directory, recursivly
// Skips entrys with an unknown extension
// Warns if an entry cannot be read for some reason
pub fn discover_source(dir: &Path) -> cu::Result<Vec<SourceFile>> {
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
        if let Some(lang) = lang {
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
pub struct SourceFile {
    path: PathBuf,
    pathhash: usize,
    basename: String,
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

    pub async fn compile(
        &self,
        flags: &Flags,
        includes: Vec<String>,
        compile_db: Arc<CompileDB>,
        output_path: &Path,
    ) -> cu::Result<Option<CompileCommand>> {
        if !output_path.exists() {
            cu::fs::make_dir(output_path).unwrap();
            cu::info!(
                "Output path {:?} exists={}",
                &output_path,
                &output_path.exists()
            );
        }
        let o_path = output_path.join(format!("{}-{}.o", self.basename, self.pathhash));
        let d_path = output_path.join(format!("{}-{}.d", self.basename, self.pathhash));

        let (comp_path, comp_flags) = match self.get_lang() {
            Lang::C => (environment().cc_path(), &flags.cflags),
            Lang::Cpp => (environment().cxx_path(), &flags.cxxflags),
            Lang::S => (environment().cc_path(), &flags.sflags),
            Lang::None => {
                return Err(cu::Error::msg(format!("Invalid extension for source file {:?}", self.path)));
            }
        };

        let comp_command = CompileCommand::new(
            comp_path, &self.path, &o_path, &d_path, comp_flags, &includes,
        )?;

        compile_db.update(comp_command.clone());

        if self.need_recompile(compile_db, &o_path, &d_path, &comp_command)? {
            // Ensure source and artifacts have the same timestamp
            let src_time = cu::fs::get_mtime(&self.path)?.unwrap();

            // Compile and update record
            comp_command.execute()?;

            cu::fs::set_mtime(o_path, src_time)?;
            if d_path.exists() {
                cu::fs::set_mtime(d_path, src_time)?;
            }
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
        let comp_record = compile_db.commands.iter().find(|command| match command {
            CompileRecord::Compile(compile_command) => compile_command.pathhash == self.pathhash,
            CompileRecord::Link(_) => false,
        });

        let comp_record = match comp_record {
            Some(CompileRecord::Compile(cmd)) => cmd,
            _ => return Ok(true),
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

        if command != comp_record {
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