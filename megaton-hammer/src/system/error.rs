//! Error types

use std::process::ExitStatus;

use buildcommon::errorln;
use error_stack::Report;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // pre-check
    #[error("`Megaton.toml` not found or an error has occured. Please run inside a Megaton project.")]
    FindProject,
    #[error("Cannot find required tool `{0}`. {1}")]
    MissingTool(String, String),
    #[error("Environment variable `{0}` is not set. {1}")]
    MissingEnv(String, String),

    // process
    #[error("error spawning `{0}`: {1}")]
    SpawnChild(String, std::io::Error),
    #[error("error executing `{0}`: {1}")]
    WaitForChild(String, std::io::Error),

    // config
    #[error("Cannot parse config file: {0}")]
    ParseConfig(String),
    #[error("Please specify a profile with `--profile`")]
    NoProfile,
    #[error("Cannot parse `{0}`: {1}")]
    ParseJson(String, serde_json::Error),
    #[error(
        "No entry point specified in the config. Please specify `entry` in the `make` section"
    )]
    NoEntryPoint,

    // build
    #[error("failed to create builder")]
    CreateBuilder,
    
    #[error("One or more object files failed to compile. Please check the errors above.")]
    CompileError,
    #[error("Linking failed. Please check the errors above")]
    LinkError,
    #[error("Invalid objdump output `{0}`: {1}")]
    InvalidObjdump(String, String),
    #[error("Objdump failed!")]
    ObjdumpFailed,
    #[error("Check failed! Check errors above.")]
    CheckError,
    #[error("Failed to convert ELF to NSO!")]
    Elf2NsoError,
    #[error("Npdmtool failed: {0}")]
    NpdmError(ExitStatus),

    #[error("Cannot build toolchain: {0}")]
    BuildToolchain(String),

    #[error("parsing regex: {0}")]
    Regex(#[from] regex::Error),

    #[cfg(windows)]
    #[error("The program is not supported on Windows.")]
    Windows,

    #[error("system error")]
    Interop(Report<buildcommon::system::Error>),

    #[error("{0}")]
    InteropSelf(Report<Self>),
}

impl Error {
    pub fn print(&self) {
        errorln!("Fatal", "{}", self);
    }
}
