// This modules handles compiling c/c++/asm/rust code

use cu::{Command, CommandBuilder, Context, Result, Spawn, args, lv, pio};
use std::path::Path;

use super::{Flags, Lang, RustCrate, SourceFile};
use crate::env::environment;

// Compiles the given source file and writes it to `out`
pub fn compile(src: &SourceFile, flags: &Flags) -> Result<()> {
    if src.up_to_date() {
        cu::debug!("{} up to date, skipping", src.path.to_str().unwrap());
        return Ok(())
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
pub fn compile_rust(rust_crate: RustCrate) -> Result<()> {
    // TODO: Implement
    Ok(todo!())
}

struct CompileCommand {
    command: Command<lv::Lv, lv::Lv, pio::Null>,
}

impl CompileCommand {
    fn new(compiler_path: &Path, src_file: &Path, out_file: &Path, flags: &Vec<String>) -> Self {
        let command = CommandBuilder::new(compiler_path)
            .stdout(lv::I)
            .stderr(lv::E)
            .stdin_null()
            .args(flags)
            .add(args!["-o", out_file, src_file,]);

        CompileCommand { command }
    }
    fn execute(self) -> Result<()> {
        self.command.spawn().context("Compiler command failed")?;
        Ok(())
    }
}
