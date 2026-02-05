// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use cu::pre::*;

// Core environment variables needed to run the tool
// Includes paths to build/debug utilities and caches
#[allow(dead_code)]
#[derive(Debug)]
pub struct Environment {
    megaton_home: PathBuf,
    devkitpro: PathBuf,
    dkp_version: String,
    dkp_includes: Vec<String>,
    libnx_include: PathBuf,
    npdmtool: PathBuf,
    elf2nso: PathBuf,
    cc: PathBuf,  // C compiler
    cxx: PathBuf, // C++ compiler
    asm: PathBuf, // Assembler
    cc_version: String,
    cxx_version: String,
    asm_version: String,
}

impl Environment {
    fn new(megaton_home: PathBuf, devkitpro: PathBuf) -> Self {
        let dkp_bin = devkitpro.join("devkitA64").join("bin");
        let cc = dkp_bin.join("aarch64-none-elf-gcc");
        let cxx = dkp_bin.join("aarch64-none-elf-g++");
        let asm = dkp_bin.join("aarch64-none-elf-gcc");
        let libnx_include = devkitpro.join("libnx").join("include");
        let dkp_tools_bin = devkitpro.join("tools").join("bin");
        let npdmtool = dkp_tools_bin.join("npdmtool");
        let elf2nso = dkp_tools_bin.join("elf2nso");
        let dkp_version = get_dkp_version(&devkitpro, &cc)
            .expect("Failed to init environment: check that DKP is installed correctly");
        let dkp_includes = get_dkp_includes(&devkitpro, &dkp_version);
        let cc_version = get_cc_version(&cc)
            .expect("Failed to init environment: error when checking compiler version");
        let cxx_version = cc_version.clone(); // These should be the same, maybe merge
        let asm_version = cc_version.clone(); // these at some point in the future

        Self {
            megaton_home,
            devkitpro,
            dkp_version,
            dkp_includes,
            libnx_include,
            npdmtool,
            elf2nso,
            cc,
            cxx,
            asm,
            cc_version,
            cxx_version,
            asm_version,
        }
    }

    /// Get the home of the megaton cache directory
    pub fn home(&self) -> &Path {
        &self.megaton_home
    }
    pub fn dkp_path(&self) -> &Path {
        &self.devkitpro
    }
    pub fn dkp_version(&self) -> &str {
        &self.dkp_version
    }
    pub fn dkp_includes(&self) -> &[String] {
        &self.dkp_includes
    }
    pub fn libnx_include(&self) -> &Path {
        &self.libnx_include
    }
    pub fn cc_path(&self) -> &Path {
        &self.cc
    }
    pub fn cxx_path(&self) -> &Path {
        &self.cxx
    }
    pub fn asm_path(&self) -> &Path {
        &self.asm
    }
    pub fn elf2nso_path(&self) -> &Path {
        &self.elf2nso
    }
    pub fn cc_version(&self) -> &str {
        &self.cc_version
    }
    pub fn cxx_version(&self) -> &str {
        &self.cxx_version
    }
    pub fn asm_version(&self) -> &str {
        &self.asm_version
    }
}

fn get_dkp_version(dkp: &Path, cc: &Path) -> cu::Result<String> {
    // /opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/
    let readdir = cu::fs::read_dir(
        dkp.join("devkitA64")
            .join("aarch64-none-elf")
            .join("include")
            .join("c++"),
    )
    .context("DKP include path does not exist")?;

    let dir = readdir.filter_map(|x| x.ok()).collect::<Vec<_>>();

    if dir.len() == 1 {
        Ok(dir[0].file_name().display().to_string())
    } else {
        // Fallback: query gcc for version
        get_cc_version(cc)
    }
}

fn get_cc_version(cc_path: &Path) -> cu::Result<String> {
    let (child, _, lines) = cu::CommandBuilder::new(cc_path.as_os_str())
        .arg("-v")
        .stdout_null()
        .stderr(cu::pio::lines())
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    let verline = lines.last().unwrap()?;
    let verstring = verline.split(" ").nth(2).unwrap().to_owned();
    Ok(verstring)
}

fn get_dkp_includes(dkp: &Path, dkp_version: &str) -> Vec<String> {
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
        dkp.join(path.replace("?ver?", dkp_version))
            .display()
            .to_string()
    })
    .collect::<Vec<_>>()
}

static ENVIRONMENT: OnceLock<Environment> = OnceLock::new();

pub fn environment() -> &'static Environment {
    ENVIRONMENT.get().expect("environment was not initialized")
}

pub fn commit() -> &'static str {
    env!("MEGATON_COMMIT")
}

/// Initialize the environment
///
/// # Safety
/// Only safe to call when only one thread exists
pub unsafe fn init_env() -> cu::Result<()> {
    let megaton_home = cu::env_var("MEGATON_HOME").unwrap_or_default();
    let megaton_home = if megaton_home.is_empty() {
        cu::debug!("MEGATON_HOME not specified, using default path ~/.cache/megaton");
        let mut home = std::env::home_dir().context("failed to get user's home directory")?;
        home.extend([".cache", "megaton"]);
        home.normalize()?
    } else {
        Path::new(&megaton_home).normalize()?
    };

    let devkitpro = cu::env_var("DEVKITPRO").context("DEVKITPRO environment variable not set")?;
    let devkitpro = Path::new(&devkitpro).normalize()?;

    cu::debug!("megaton_home: {}", megaton_home.display());

    let env = Environment::new(megaton_home, devkitpro);
    if ENVIRONMENT.set(env).is_err() {
        cu::bail!("unexpected: environment was already set before init_env()");
    }

    Ok(())
}
