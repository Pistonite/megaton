// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std:: path::{Path, PathBuf} ;

use cu::pre::*;

use super::compile_db::CompileRecord;

use crate::buildsys::compile::SourceType;
use crate::env::Environment;
use crate::config::Flags;

/// A source file and its corresponding artifacts
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub pathhash: usize,
    basename: String,
    typ: SourceType,
}

#[derive(PartialEq, Eq, Debug)]
pub enum SourceStatus {
    /// The source doesn't need compiled, its artifact is at `path`
    UpToDate(PathBuf),
    /// The source needs to be compiled with `record`
    CompileNeeded(CompileRecord),
}



impl SourceFile {
    /// Returns a source file object, or None, if the given file is not a valid source
    /// i.e. the name cannot be parsed or the file extension is invalid
    fn from_path(path: PathBuf) -> Option<Self> {
        // if path does not have extension or does not 
        let source_type = SourceType::from_extension(path.extension()?)?;
        let basename = match path.file_name()?.as_utf8() {
            Err(e) => {
                cu::warn!("skipping source with non-utf8 name: {e}");
                return None
            }
            Ok(x) => x
        };

        Some(Self {
            pathhash: fxhash::hash(&path),
            basename: basename.to_string(),
            path,
            typ: source_type,
        })
    }

    pub fn configure(
        self,
        flags: &Flags,
        output_path: &Path,
        record: Option<&CompileRecord>,
        env: &'static Environment
    ) -> cu::Result<SourceStatus> {

        let compiler = self.typ.get_compiler(env);
        // FIXME: clone() is expensive to do for every source file
        let mut args = self.typ.get_flags(flags).clone();

        let o_path = self.get_o_path(output_path);
        let d_path = self.get_d_path(output_path);

        // Add arguments to generate depfile
        args.push("-MMD".to_string());
        args.push("-MP".to_string());
        args.push("-MF".to_string());
        args.push(d_path.display().to_string());

        args.push("-c".to_string());
        args.push(format!("-o{}", o_path.display()));
        args.push(self.path.display().to_string());

        if self.up_to_date(record, output_path, &args)? {
            cu::debug!("Compile: object up to date {}", o_path.display());
            Ok(SourceStatus::UpToDate(o_path))
        } else {
            Ok(SourceStatus::CompileNeeded(CompileRecord {
                source_path: self.path,
                compiler: compiler.to_owned(),
                args,
                o_path,
                d_path,
            }))
        }
    }

    fn up_to_date(
        &self,
        record: Option<&CompileRecord>,
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
        if !o_path.exists() || (!d_path.exists() && self.typ.uses_depfile() ) {
            return Ok(false);
        }

        // Check that source time = o time
        if cu::fs::get_mtime(&o_path)? != cu::fs::get_mtime(&self.path)? {
            return Ok(false);
        }

        // Assembly files don't need dependency checks
        if !self.typ.uses_depfile() {
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
                cu::bail!(
                    "Error when parsing depfile {} at position {pos}",
                    d_path.display()
                );
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

    fn get_o_path(&self, output_path: &Path) -> PathBuf {
        output_path.join(format!("{}-{:016x}.o", self.basename, self.pathhash))
    }

    fn get_d_path(&self, output_path: &Path) -> PathBuf {
        output_path.join(format!("{}-{:016x}.d", self.basename, self.pathhash))
    }
}



pub fn scan(dirs: &[PathBuf]) -> cu::Result<impl Iterator<Item=SourceFile>> {
    cu::debug!("dirs to scan: {dirs:#?}");
    let mut walks = Vec::with_capacity(dirs.len());
    for dir in dirs {
        let walk = cu::check!(cu::fs::walk(dir), "failed to open source directory")?;
        walks.push(WalkIterAdapter{walk});
    }
    struct WalkIterAdapter {
        walk: cu::fs::Walk,
    }

    impl Iterator for WalkIterAdapter {
        type Item = SourceFile;
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let entry = match self.walk.next()? {
                    Ok(x) => x,
                    Err(e) => {
                        cu::warn!("error reading source directory entry: {e:?}");
                        continue;
                    }
                };
                if let Some(x) = SourceFile::from_path(entry.path()) {
                    return Some(x);
                }
            }
        }
    }

    Ok(walks.into_iter().flatten())
}

