use std::{path::{Path, PathBuf}, process::ExitStatus};
use cu::{Child, Spawn, str::PathExtension};
use serde::{Deserialize, Serialize};
use crate::env::environment;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct CompileCommand {
    pub pathhash: usize,
    pub compiler: PathBuf,
    pub source: PathBuf,
    pub args: Vec<String>,
    pub sys_headers: Vec<String>,
    pub output_file: PathBuf,
    pub dep_file: PathBuf
}

impl CompileCommand {
    pub fn new(
        compiler_path: &Path,
        src_file: &Path,
        out_file: &Path,
        dep_file: &Path,
        flags: &[String],
        includes: &[String],
    ) -> Self {
        let mut args = flags.to_owned();
        args.push("-MMD".to_owned());
        args.push("-MP".to_owned());
        args.push("-MF".to_owned());
        args.push(dep_file.display().to_string());

        let includes = includes
            .iter()
            .filter_map(|i| {
                let path = PathBuf::from(i);
                path.canonicalize()
                    .inspect_err(|e| cu::error!("cant find include {} {}", path.display(), e))
                    .ok()
            })
            .map(|i| format!("-I{}", i.as_os_str().to_str().unwrap()))
            .collect::<Vec<String>>();

        args.extend(includes);

        args.push(String::from("-c"));

        args.push(format!("-o{}", out_file.display()));

        args.push(src_file.display().to_string());

        cu::trace!(
            "Compiler command: \n{} {} {}",
            &compiler_path.display(),
            &src_file.display(),
            &args.join(" ")
        );

        let src_path = src_file.to_path_buf();
        Self {
            pathhash: fxhash::hash(&src_path),
            compiler: compiler_path.to_path_buf(),
            source: src_path,
            args,
            sys_headers: devkitpro_includes(),
            output_file: out_file.to_path_buf(),
            dep_file: dep_file.to_path_buf()
        }
    }

    pub async fn execute(&self) -> cu::Result<Child> {
        cu::trace!(
            "Executing CompileCommand: \n{} {}",
            &self.compiler.display(),
            &self.args.join(" ")
        );

        let child  = self.compiler.command()
            .args(self.args.clone())
            .stdoe(cu::pio::inherit()) // todo: log to file
            .stdin_null()
            .co_spawn().await?;
        
        Ok(child)
    }

    fn command(&self) -> String {
        format!("{} {}", self.compiler.display(), self.args.join(" "))
    }
}

pub fn devkitpro_includes() -> Vec<String> {
    [
        "devkitA64/aarch64-none-elf/include/c++/?ver?",
        "devkitA64/aarch64-none-elf/include/c++/?ver?/aarch64-none-elf",
        "devkitA64/aarch64-none-elf/include/c++/?ver?/backward",
        "devkitA64/lib/gcc/aarch64-none-elf/?ver?/include",
        "devkitA64/lib/gcc/aarch64-none-elf/?ver?/include-fixed",
        "devkitA64/aarch64-none-elf/include",
    ]
    .iter()
    .map(|path| {
        environment()
            .dkp_path()
            .join(path.replace("?ver?", environment().dkp_version()))
            .display()
            .to_string()
    })
    .collect::<Vec<_>>()
}