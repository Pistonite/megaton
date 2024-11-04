#[derive(Debug, thiserror::Error)]
pub enum Error {
    // install
    #[error("failed to install or update build tool")]
    Install,
    #[error("failed to update megaton repo")]
    InstallUpdate,
    #[error("failed to run cargo build while installing")]
    InstallCargoBuild,
    #[error("failed to replace executable while installing")]
    ReplaceExe,
    #[error("failed to create shim for build tool")]
    CreateShim,
    #[error("run `megaton install` to complete installation")]
    NeedRerun,

    // checkenv
    #[error("environment check failed")]
    CheckEnv,

    // clean
    #[error("failed to clean")]
    Clean,

    // build:config
    #[error("failed to load project config")]
    Config,
    #[error("no profile selected")]
    NoProfile,
    #[error("no entry point specified")]
    NoEntryPoint,

    // build:prep
    #[error("error when preparing build")]
    BuildPrep,
    #[error("error when processing source files")]
    SourcePrep,

    // build:make
    #[error("failed to compile")]
    Compile,
    #[error("failed to link")]
    Link,
    #[error("failed to process linker script")]
    Ldscript,

    // build:check
    #[error("failed to create checker")]
    CreateChecker,
    #[error("failed to parse symbols from {0}")]
    ParseSymbols(String),
    #[error("failed to execute objdump -T")]
    ObjdumpSymbols,
    #[error("failed to execute objdump -p")]
    ObjdumpInstructions,
    #[error("failed to parse instruction regex")]
    ParseInstRegex,
    #[error("errors found when checking ELF")]
    CheckError,

    // build:other tools/outputs
    #[error("failed to create compile_commands.json")]
    CompileDb,
    #[error("failed to create verfile")]
    Verfile,
    #[error("failed to create npdm file")]
    Npdm,
    #[error("failed to convert ELF to NSO!")]
    Elf2Nso,
}
