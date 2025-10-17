// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use cu::fs::{WalkEntry, walk};
use cu::{args, lv, pio, Command, CommandBuilder, Context, Result, Spawn};

use super::Flags;
use crate::env::environment;

pub fn scan_and_compile_sources(sources: &Vec<String>, build_flags: &Flags) -> Result<()> {
    for dir in sources {
        match walk(Path::new(dir)) {
            Ok(mut walk) => {
                while let Some(walk_result) = walk.next() {
                    match walk_result {
                        Ok(entry) => {
                            if needs_recompile(&entry) {
                                compile_src(entry.path(), build_flags)?;
                            }
                        }
                        Err(_) => {
                            cu::warn!("Failed to read entry while walking {dir}")
                        }
                    };
                }
            }
            Err(e) => {
                cu::warn!("Failed to walk {dir}: {e}")
            }
        }
    }

    Ok(())
}

fn compile_src(src: PathBuf, build_flags: &Flags) -> Result<()> {
    let src_file = OsStr::new(&src);
    let mut out = src.clone();
    out.set_extension("o");
    let out_file = OsStr::new(&out);

    let extention = src.extension().and_then(OsStr::to_str).unwrap_or_default();
    match extention {
        "c" => {
            cu::info!("Compiling {} with gcc", src_file.to_str().unwrap());
            let command = CompileCommand::new(Compiler::Cc, src_file, out_file, &build_flags.cflags);
            command.execute()?;
        }
        "cc" | "cpp" | "cxx" | "c++" => {
            cu::info!("Compiling {} with g++", src_file.to_str().unwrap());
            let command = CompileCommand::new(Compiler::Cxx, src_file, out_file, &build_flags.cxxflags);
            command.execute()?;
        }
        "s" | "asm" => {
            cu::warn!("Implement assembler to build .s and .asm files");
        }
        _ => {
            cu::debug!(
                "{}: Unknown file extension - skipping",
                src.to_str().unwrap()
            );
        }
    };

    Ok(())
}

fn needs_recompile(_file: &WalkEntry) -> bool {
    // TODO: Calculuate if recomiple is required
    true
}

enum Compiler {
    Cc,
    Cxx,
}

struct CompileCommand {
    command: Command<lv::Lv, lv::Lv, pio::Null>
}

impl CompileCommand {
    fn new(compiler: Compiler, src_file: &OsStr, out_file: &OsStr, flags: &Vec<String>) -> Self {
        let compiler_path = match compiler {
            Compiler::Cc => environment().cc_path(),
            Compiler::Cxx => environment().cxx_path(),
        };
        let command = CommandBuilder::new(compiler_path)
            .stdout(lv::I)
            .stderr(lv::E)
            .stdin_null()
            .args(flags)
            .add(args![
                "-o", out_file,
                src_file,
            ]);

        CompileCommand { command }
    }
    fn execute(self) -> Result<()> {
        self.command.spawn().context("Compiler command failed")?;
        Ok(())
    }
}
