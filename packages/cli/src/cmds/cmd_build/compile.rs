use std::path::Path;

use cu::pre::*;

use super::{Flags, Lang, RustCrate, SourceFile};
use crate::env::environment;

// Compiles the given source file and writes it to `out`
pub fn compile(src: &SourceFile, flags: &Flags) -> cu::Result<()> {
    if src.up_to_date() {
        cu::debug!("{} up to date, skipping", src.path.to_str().unwrap());
        return Ok(());
    }
    let (comp_path, comp_flags) = match src.lang {
        Lang::C => (environment().cc_path(), &flags.cflags),
        Lang::Cpp => (environment().cxx_path(), &flags.cxxflags),
        Lang::S => (environment().cc_path(), &flags.sflags),
    };

    let comp_command = CompileCommand::new(comp_path, &src.path, &src.o_path, comp_flags);

    comp_command.execute()?;

    Ok(())
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
    fn new(compiler_path: &Path, src_file: &Path, out_file: &Path, flags: &Vec<String>) -> Self {
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
