// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use cu::pre::*;

use super::compile_db::CompileRecord;

use crate::config::Flags;
use crate::env;

/// A source file and its corresponding artifacts
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub pathhash: usize,
    basename: String,
    lang: Lang,
}

/// UpToDate(path) - The source doesn't need compiled, its artifact is at `path`
/// CompileNeeded(record) - The source needs to be compiled with `record`
#[derive(PartialEq, Eq, Debug)]
pub enum SourceStatus {
    UpToDate(PathBuf),
    CompileNeeded(CompileRecord),
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Lang {
    C,
    Cpp,
    S,
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

    pub fn configure_compilation(
        self,
        flags: &Flags,
        output_path: &Path,
        record: Option<&CompileRecord>,
    ) -> cu::Result<SourceStatus> {
        let compiler = self.lang.get_compiler_path();
        let mut args = self.lang.get_flags(flags).clone();

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

impl Lang {
    fn get_compiler_path(&self) -> &Path {
        let env = env::get();
        match self {
            Lang::C => env.cc(),
            Lang::Cpp => env.cxx(),
            Lang::S => env.cc(),
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

pub fn scan(dirs: &[PathBuf]) -> SourceIterator {
    let walks = dirs
        .iter()
        .filter_map(|dir| cu::fs::walk(dir).ok())
        .collect::<Vec<_>>();

    SourceIterator { walks }
}

pub struct SourceIterator {
    walks: Vec<cu::fs::Walk>,
}

impl Iterator for SourceIterator {
    type Item = SourceFile;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut walk) = self.walks.pop() {
            while let Some(Ok(entry)) = walk.next() {
                let source = SourceFile::new(entry.path());
                if let Some(source) = source {
                    self.walks.push(walk);
                    cu::debug!("Scan: found source {}", source.path.display());
                    return Some(source);
                }
            }
        }
        None
    }
}
