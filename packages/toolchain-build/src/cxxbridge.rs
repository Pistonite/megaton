use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::home;

/// The "blessed" cxxbridge-cmd version to use
pub static BLESSED_VERSION: &str = "1.0.194";

pub struct CxxBridgeInfo {
    pub version: String,
}

/// Check if cxxbridge tool is installed
pub fn check(home: &Path, print: bool) -> cu::Result<Option<CxxBridgeInfo>> {
    let bin_path = binary_path_internal(home);
    let (child, out) = bin_path
        .command()
        .arg("--version")
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Ok(None);
    }
    let version_string = out.join()??;
    if print {
        cu::print!("{version_string}");
        cu::print!("location: {}", bin_path.display());
    }
    let version = match version_string.strip_prefix("cxxbridge ") {
        Some(v) => v.trim(),
        None => {
            cu::warn!("failed to parse cxxbridge version: {version_string}, assuming blessed");
            BLESSED_VERSION
        }
    };
    Ok(Some(CxxBridgeInfo {
        version: version.to_string(),
    }))
}

/// Get location of the cxxbridge tool and check if it exists
pub fn binary_path(home: &Path) -> cu::Result<PathBuf> {
    let p = binary_path_internal(home);
    if !p.exists() {
        cu::bail!("cxxbridge is not installed. please run `megaton toolchain install`");
    }
    Ok(p)
}

/// Location of the cxxbridge tool
fn binary_path_internal(home: &Path) -> PathBuf {
    home::get_bin_path(home, "cxxbridge")
}

/// Install cxxbridge tool as part of the megaton toolchain
pub fn install(home: &Path) -> cu::Result<()> {
    match check(home, false) {
        Err(e) => {
            cu::debug!("error with checking cxxbridge-cmd version: {e}, proceeding with reinstall");
        }
        Ok(None) => {
            cu::debug!("no cxxbridge-cmd installation found");
        }
        Ok(Some(info)) => {
            if info.version != BLESSED_VERSION {
                cu::debug!("cxxbridge-cmd version doesn't match blessed version, reinstalling");
            } else {
                cu::hint!("found existing cxxbridge-cmd installation, skipping");
                return Ok(());
            }
        }
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
            "--root",
            home,
            "--no-track",
            "-q", // suppress warning about adding ./bin to path
            format!("cxxbridge-cmd@{BLESSED_VERSION}"),
        ])
        .preset(cu::pio::cargo("installing cxxbridge-cmd"));

    command.spawn()?.0.wait_nz()
}

/// Uninstall cxxbridge tool
pub fn remove(home: &Path) -> cu::Result<()> {
    cu::fs::remove(binary_path_internal(home))
}
