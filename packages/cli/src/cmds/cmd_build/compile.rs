// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::path::Path;

use cu::pre::*;

use super::{Flags, Lang, RustCrate, SourceFile};
use crate::env::environment;

pub fn need_to_recompile(src, compdb) -> Option<comp_record> {
    let o_file = get_object(src);
    if o_file == None {
        return true;
    } 
    
    let d_file = get_depfile(src);
    if d_file == None {
        return true;
    } 

    if compdb[src.name].compiletime < src.metadata.modifiedtime {
        return true;
    }

    if compdb[src.name].compiletime < o_file.metadata.modifiedtime {
        return true;
    }

    if compdb[src.name].compiletime < d_file.metadata.modifiedtime {
        return true;
    }
    
    for dep in d_file {
        if compdb[src.name].compiletime < dep.metadata.modifiedtime {
            return true;
        }
    } 

    let comp_comm = CompileCommand::new();

    if compdb[src.name].command != comp_comm {
        return true
    }

    if compdb.comp_ver != environment.comp_ver {
        return true
    }

    false
}

// Compiles the given source file and writes it to `out`
pub fn compile(src: &SourceFile, flags: &Flags) -> cu::Result<()> {


    if let Some(cc) = need_to_recompile(, compdb) {
        let timefinished = cc.execute()
        compdb.update(cc, timefinsihed)
    }






}

// Builds the give rust crate and places the binary in the target as specified in the rust manifest
pub fn compile_rust(rust_crate: RustCrate) -> cu::Result<()> {
    // TODO: Implement
    Ok(todo!())
}

struct CompileCommand {
    command: cu::Command<cu::lv::Lv, cu::lv::Lv, cu::pio::Null>,
}

impl CompileCommand {
    fn new(compiler_path: &Path, src_file: &Path, out_file: &Path, flags: &Vec<String>, build_env) -> Self {
        let command = cu::CommandBuilder::new(compiler_path)
            .stdout(cu::lv::I)
            .stderr(cu::lv::E)
            .stdin_null()
            .args(flags)
            .add(cu::args!["-o", out_file, src_file,]);

        CompileCommand { command }
    }
    fn execute(self) -> cu::Result<()> {
        self.command.spawn().context("Compiler command failed")?;
        Ok(())
    }
}
