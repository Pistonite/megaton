// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::path::{Path, PathBuf};

use cu::pre::*;

/// The Rust toolchain name to be installed/linked
static TOOLCHAIN_NAME: &str = "megaton";
/// The Rust compiler repo
static RUST_REPO: &str = "https://github.com/rust-lang/rust";
/// The "blessed" commit hash to use (i.e. tested and will work)
pub static BLESSED_COMMIT: &str = "0d31508599a7814a7044e9a7a871e3dc5f037753";
/// The "blessed" version tag corresponding to the commit
pub static BLESSED_VERSION: &str = "1.100.0-dev";
/// Rust compiler/library patch file
static RUST_PATCH: &[u8] = include_bytes!("../scripts/rust-2026-09-09-0d31508599a7814a.patch");
/// File name the patch is written to inside the rust repo, to be applied from there
static RUST_PATCH_FILE: &str = "megaton.patch";

pub struct RustToolchainInfo {
    pub commit_hash: Option<String>,
}

/// Check the current status of the megaton Rust toolchain
pub fn check(print: bool) -> cu::Result<Option<RustToolchainInfo>> {
    let (child, out) = cu::which("rustc")?
        .command()
        .arg(format!("+{TOOLCHAIN_NAME}"))
        .arg("-vV")
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    let status = child.wait()?;
    if !status.success() {
        // toolchain not found
        return Ok(None);
    }
    let output = out.join()??;
    let mut commit_hash = None;
    for line in output.lines() {
        if print {
            cu::print!("{line}");
        }
        if let Some(line) = line.strip_prefix("commit-hash:") {
            let line = line.trim();
            if line == "unknown" || line.is_empty() {
                // no commit hash info for the toolchain installation
                continue;
            }
            commit_hash = Some(line.to_string());
        }
    }
    Ok(Some(RustToolchainInfo { commit_hash }))
}

pub fn install(home: &Path, keep: bool, mut clean: bool) -> cu::Result<()> {
    cu::check!(
        cu::which("rustup"),
        "rustup is required to manage Rust toolchains. Please install Rust"
    )?;
    cu::check!(
        cu::which("rustc"),
        "rustc is required to manage Rust toolchains. Please install Rust"
    )?;

    // check current status
    match check(false) {
        Err(e) => {
            cu::debug!("failed to check existing toolchain status: {e}, proceeding with reinstall");
        }
        Ok(None) => {
            cu::debug!("no toolchain installation found");
        }
        Ok(Some(info)) => {
            match info.commit_hash {
                None => {
                    cu::warn!("existing toolchain found with unknown commit hash");
                    match cu::yesno!("do you want to reinstall from the blessed commit?") {
                        Err(e) => {
                            cu::hint!(
                                "prompt is disabled. you can remove the toolchain with 'megaton toolchain remove', then try install again."
                            );
                            cu::rethrow!(e);
                        }
                        Ok(false) => {
                            cu::bail!("aborted by user");
                        }
                        Ok(true) => {}
                    }
                }
                Some(hash) => {
                    if hash == BLESSED_COMMIT {
                        cu::warn!(
                            "found existing toolchain installation that matched blessed commit hash"
                        );
                        cu::hint!(
                            "if you want to reinstall it, remove it first with 'megaton toolchain remove'."
                        );
                        return Ok(());
                    }
                    // older toolchain installed, proceed with reinstallment
                    cu::info!("found older toolchain installation with commit hash: {hash}");
                    remove(home)?;
                }
            }
        }
    }

    cu::check!(cu::which("git"), "git is required to clone rust source")?;
    cu::check!(cu::which("ninja"), "ninja is required to build llvm")?;
    cu::check!(cu::which("cmake"), "cmake is required to build llvm")?;

    let host_triple = get_rustc_host_triple()?;
    cu::info!("building rust toolchain for host triple: {host_triple}");
    let rust_path = source_location(home);
    if !clean {
        // try to get the current commit hash, will succeed if we have a valid repo
        // note we perform a checkout_blessed_commit regardless to re-apply the patch if needed
        match try_get_rust_source_commit(&rust_path) {
            Ok(hash) => {
                cu::debug!("current commit: {hash}");
                if hash != BLESSED_COMMIT {
                    cu::info!(
                        "current commit is not the blessed commit, checking out the blessed commit"
                    );
                }
            }
            Err(e) => {
                cu::debug!("cannot get current commit: {e}");
                clean = true;
            }
        }
    }
    if clean {
        cu::warn!("performing full re-checkout");
        clone_rust_source(&rust_path)?;
    }
    checkout_blessed_commit(&rust_path)?;

    // verify the blessed commit is checked out
    let actual_commit = cu::check!(
        try_get_rust_source_commit(&rust_path),
        "failed to verify the blessed commit is checked out"
    )?;
    if actual_commit != BLESSED_COMMIT {
        cu::bail!("failed to checkout the blessed commit.");
    }

    let install_location = install_location(home);
    cu::fs::make_dir_empty(&install_location)?;
    let install_location = install_location.normalize_exists()?;

    let bootstrap_toml = {
        let bootstrap_toml_template =
            cu::fs::read_string(rust_path.join("bootstrap.megaton.toml"))?;
        let llvm_targets = if host_triple.starts_with("x86_64-") {
            "llvm.targets = 'AArch64;X86'"
        } else if host_triple.starts_with("aarch64-") {
            "llvm.targets = 'AArch64'"
        } else {
            cu::warn!("using default llvm targets since the host is neither x86_64 or aarch64");
            ""
        };
        let install_prefix = install_location.as_utf8()?;
        let install_sysconfdir = install_location.join("etc").into_utf8()?;
        let rust_lto = if host_triple == "x86_64-unknown-linux-gnu" {
            // enable thin lto on x86 linux gnu, which is the only target tested
            "rust.lto = 'thin'"
        } else {
            ""
        };
        let mut bootstrap_toml = String::new();
        let mut start = 0;
        while let Some(i) = bootstrap_toml_template[start..].find("{{MEGATON_") {
            let i = start + i;
            bootstrap_toml.push_str(&bootstrap_toml_template[start..i]);
            let Some(j) = bootstrap_toml_template[i..].find("}}") else {
                cu::bail!(
                    "unexpected: unclosed {{{{MEGATON_ key in bootstrap template; this is a bug in megaton"
                );
            };
            start = i + j + 2;
            match &bootstrap_toml_template[i..start] {
                "{{MEGATON_LLVM_TARGETS_KEY_VALUE}}" => {
                    bootstrap_toml.push_str(llvm_targets);
                }
                "{{MEGATON_HOST_TRIPLE}}" => {
                    bootstrap_toml.push_str(&host_triple);
                }
                "{{MEGATON_INSTALL_PREFIX}}" => {
                    bootstrap_toml.push_str(install_prefix);
                }
                "{{MEGATON_INSTALL_SYSCONFDIR}}" => {
                    bootstrap_toml.push_str(&install_sysconfdir);
                }
                "{{MEGATON_RUST_LTO_KEY_VALUE}}" => {
                    bootstrap_toml.push_str(rust_lto);
                }
                other => {
                    cu::bail!(
                        "unexpected: unknown '{other}' key in bootstrap template; this is a bug in megaton"
                    );
                }
            }
        }
        bootstrap_toml.push_str(&bootstrap_toml_template[start..]);
        cu::trace!("bootstrap_toml: {bootstrap_toml}");
        bootstrap_toml
    };

    cu::fs::write(rust_path.join("bootstrap.toml"), bootstrap_toml)?;

    cu::info!("building and installing rust");
    cu::hint!(" ** this may take a while, please be patient **");
    {
        let debug_log = cu::lv::D.enabled();
        let command = cu::bin::resolve("rust-x", rust_path.join("x"))?
            .command()
            .current_dir(&rust_path)
            .add(cu::color_flag())
            .args(["--stage", "1", "install", "compiler/rustc", "library/std"])
            .stdin_null();
        let (code, bar) = if debug_log {
            (command.stdoe(cu::pio::inherit()).wait()?, None)
        } else {
            let (child, bar, _) = command.stdoe(cu::pio::spinner("")).spawn()?;
            (child.wait()?, Some(bar))
        };

        if !code.success() {
            if !debug_log {
                cu::hint!("enable verbose output -v to see the output from rust/x");
            }
            cu::bail!("rust/x failed!");
        }

        if let Some(bar) = bar {
            bar.done();
        }
    }

    let install_location = self::install_location(home);
    let toolchain_link_result = cu::which("rustup")?
        .command()
        .add(cu::args![
            "toolchain",
            "link",
            TOOLCHAIN_NAME,
            install_location
        ])
        .all_null()
        .wait_nz();
    cu::check!(toolchain_link_result, "failed to link built toolchain")?;

    let toolchain_info = cu::check!(
        check(true),
        "failed to get toolchain, installation might have failed."
    )?;
    let toolchain_info = cu::check!(
        toolchain_info,
        "failed to get toolchain, installation might have failed."
    )?;
    match toolchain_info.commit_hash {
        None => {
            cu::warn!("failed to get commit hash from installed toolchain");
        }
        Some(hash) => {
            if hash == BLESSED_COMMIT {
                cu::info!("verified installed toolchain has the blessed commit hash");
            } else {
                cu::warn!("the installed toolchain does not have the blessed commit hash");
            }
        }
    }

    if !keep {
        cu::info!("removing build artifacts to free disk space");
        cu::hint!("- use --keep if you want to keep them");
        let _bar = cu::progress("removing build artifacts");
        cu::fs::rec_remove(rust_path)?;
    } else {
        cu::hint!("keeping build artifacts since --keep is specified");
    }

    cu::info!("toolchain installed successfully!");

    Ok(())
}

pub fn remove(home: &Path) -> cu::Result<()> {
    cu::which("rustup")?
        .command()
        .args(["toolchain", "uninstall", TOOLCHAIN_NAME])
        .name("rustup")
        .stderr(cu::lv::P)
        .stdio_null()
        .wait_nz()?;
    if let Ok(Some(_)) = check(false) {
        cu::bail!(
            "failed to uninstall toolchain. Please run 'rustup toolchain uninstall {TOOLCHAIN_NAME}' to uninstall it manually, then try again."
        );
    }

    let install_path = install_location(home);
    cu::debug!(
        "cleaning up toolchain files at '{}'",
        install_path.display()
    );
    cu::check!(
        cu::fs::make_dir_empty(install_path),
        "failed to clean up old toolchain files"
    )?;
    Ok(())
}

pub fn clean(home: &Path) -> cu::Result<()> {
    let rust_path = source_location(home);
    if !rust_path.exists() {
        cu::info!("rust repo is already removed");
        return Ok(());
    }
    let _bar = cu::progress("removing rust repo");
    if let Err(e) = cu::fs::rec_remove(rust_path) {
        cu::warn!("failed to remove rust repo: {e}");
    }
    Ok(())
}

fn install_location(home: &Path) -> PathBuf {
    home.join("rust-toolchain")
}

fn source_location(home: &Path) -> PathBuf {
    home.join("rust")
}

/// Get rustc host triple by running `rustc -vV`, like `x86_64-unknown-linux-gnu`
fn get_rustc_host_triple() -> cu::Result<String> {
    let (child, output) = cu::which("rustc")?
        .command()
        .arg("-vV")
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let output = output.join()??;
    for line in output.lines() {
        if let Some(host) = line.strip_prefix("host: ") {
            return Ok(host.to_string());
        }
    }
    cu::bail!("failed to get host triple from rustc");
}

fn clone_rust_source(path: &Path) -> cu::Result<()> {
    cu::check!(
        cu::fs::make_dir_empty(path),
        "fail to clean rust source directory"
    )?;
    let git = cu::which("git")?;
    git.command()
        .add(cu::args!["-C", &path, "init"])
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    git.command()
        .add(cu::args!["-C", &path, "remote", "add", "origin", RUST_REPO])
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    Ok(())
}

fn checkout_blessed_commit(path: &Path) -> cu::Result<()> {
    let git = cu::which("git")?;
    let (child, bar, _) = git
        .command()
        .add(cu::args![
            "-C",
            &path,
            "fetch",
            "origin",
            BLESSED_COMMIT,
            "--progress",
            "--depth",
            "1"
        ])
        .stdoe(cu::pio::spinner("fetching rust source"))
        .stdin_null()
        .spawn()?;
    // git doesn't exit with 0..
    child.wait()?;
    bar.done();

    // --force, since the patch applied to the previously checked-out commit
    // leaves the tracked files modified
    let (child, bar, _) = git
        .command()
        .add(cu::args![
            "-C",
            &path,
            "checkout",
            "--force",
            BLESSED_COMMIT,
            "--progress"
        ])
        .stdoe(cu::pio::spinner("checking-out rust source"))
        .stdin_null()
        .spawn()?;
    // git doesn't exit with 0..
    child.wait()?;
    bar.done();

    // delete untracked files
    let clean_result = git
        .command()
        .add(cu::args!["-C", &path, "clean", "--force", "-d"])
        .stdoe(cu::lv::D)
        .stdin_null()
        .wait_nz();
    cu::check!(clean_result, "failed to clean the rust source")?;

    // apply the megaton patch
    cu::fs::write(path.join(RUST_PATCH_FILE), RUST_PATCH)?;
    let apply_result = git
        .command()
        .add(cu::args![
            "-C",
            &path,
            "apply",
            "--whitespace=nowarn",
            RUST_PATCH_FILE
        ])
        .stdoe(cu::pio::spinner("applying megaton rust patch"))
        .stdin_null()
        .spawn();
    let (child, bar, _) = cu::check!(apply_result, "failed to apply the rust patch: spawn failed")?;
    cu::check!(child.wait_nz(), "failed to apply the rust patch")?;
    bar.done();

    Ok(())
}

/// Get the rust source commit of the currently checked out rust repo,
/// if any
fn try_get_rust_source_commit(path: &Path) -> cu::Result<String> {
    if !path.join(".git").exists() {
        cu::bail!("not a git repo: '{}'", path.display());
    }
    let (child, commit) = cu::which("git")?
        .command()
        .add(cu::args!["-C", &path, "rev-parse", "HEAD"])
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let commit = commit.join()??;
    let commit = commit.trim();
    if commit.is_empty() {
        cu::bail!("commit is empty");
    }
    Ok(commit.to_string())
}
