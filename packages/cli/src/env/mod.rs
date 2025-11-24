// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use cu::pre::*;

// Core environment variables needed to run the tool
// Includes paths to build/debug utilities and caches
#[derive(Debug)]
pub struct Environment {
    megaton_home: PathBuf,
    devkitpro: PathBuf,
    cc: PathBuf,  // C compiler
    cxx: PathBuf, // C++ compiler
    asm: PathBuf, // Assembler
    libnx_include: PathBuf,
    npdmtool: PathBuf,
    elf2nso: PathBuf,
}

impl Environment {
    fn new(megaton_home: PathBuf, devkitpro: PathBuf) -> Self {
        let dkp_bin = devkitpro.join("devkitA64").join("bin");
        let cc = dkp_bin.join("aarch64-none-elf-gcc");
        let cxx = dkp_bin.join("aarch64-none-elf-g++");
        let asm = dkp_bin.join("aarch64-none-elf-gcc"); // Use gcc for now
        let libnx_include = devkitpro.join("libnx").join("include");
        let dkp_tools_bin = devkitpro.join("tools").join("bin");
        let npdmtool = dkp_tools_bin.join("npdmtool");
        let elf2nso = dkp_tools_bin.join("elf2nso");

        Self {
            megaton_home,
            devkitpro,
            cc,
            cxx,
            asm,
            libnx_include,
            npdmtool,
            elf2nso,
        }
    }

    /// Get the home of the megaton cache directory
    pub fn home(&self) -> &Path {
        &self.megaton_home
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
