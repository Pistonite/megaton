// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use cu::pre::*;
use semver::Version;

use megaton_config::toolchain::{CxxBridgeEnv, HomeEnv};

/// The "blessed" cxxbridge-cmd version to use
pub static BLESSED_VERSION: &str = "1.0.197";

/// Get the blessed cxx version as a semver
pub fn blessed_semver() -> Version {
    Version::parse(BLESSED_VERSION).expect("failed to parse blessed cxx version")
}

pub fn inspect(home: &HomeEnv) {
    let env = CxxBridgeEnv::resolve(home);
    if !env.is_installed() {
        cu::print!("cxxbridge: <not installed>");
        return;
    }
    cu::print!("cxxbridge: {} ({})", env.version, env.path.display());
}

/// Install cxxbridge tool
#[cu::context("failed to install cxxbridge-cmd")]
pub fn install(home: &HomeEnv) -> cu::Result<()> {
    let env = CxxBridgeEnv::resolve(home);
    if env.is_installed() {
        if env.version != BLESSED_VERSION {
            let current  = &env.version;
            cu::debug!("cxxbridge-cmd version {current} doesn't match blessed version {BLESSED_VERSION}, reinstalling");
        } else {
            cu::hint!("found existing cxxbridge-cmd installation, skipping");
            return Ok(());
        }
    } else {
        cu::debug!("no cxxbridge-cmd installation found");
    }
    let cargo = cu::check!(
        cu::which("cargo"),
        "cargo is required to build cxxbridge; please install Rust"
    )?;
    // using home as root so the bin is installed to <home>/bin
    let command = cargo
        .command()
        .add(cu::args![
            "install",
            "--force",
            "--root",
            &home.path,
            "--no-track",
            "-q", // suppress warning about adding ./bin to path
            format!("cxxbridge-cmd@{BLESSED_VERSION}"),
        ])
        .preset(cu::pio::cargo("installing cxxbridge-cmd"));

    command.spawn()?.0.wait_nz()
}

/// Uninstall cxxbridge tool
#[cu::context("failed to uninstall cxxbridge-cmd")]
pub fn remove(home: &HomeEnv) -> cu::Result<()> {
    let env = CxxBridgeEnv::resolve(home);
    cu::fs::remove(env.path)
}

pub fn invoke(home: &HomeEnv, args: Vec<String>) -> cu::Result<()> {
    let env = CxxBridgeEnv::resolve(home);
    env.path.command().args(args)
    .all_inherit().wait_nz()
}
