// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::path::Path;

use cu::pre::*;
use megaton_config::toolchain::{HomeEnv, MEGATON_TOOLCHAIN_NAME, RustEnv, RustEnvPartial, RustHostEnv};

/// The Rust compiler repo
static RUST_REPO: &str = "https://github.com/rust-lang/rust";
/// The "blessed" commit hash to use (i.e. tested and will work)
pub static BLESSED_COMMIT: &str = "0d31508599a7814a7044e9a7a871e3dc5f037753";
/// The "blessed" version tag corresponding to the commit
#[allow(unused)]
pub static BLESSED_VERSION: &str = "1.100.0-dev";
/// Rust compiler/library patch file
static RUST_PATCH: &[u8] = include_bytes!("../scripts/rust-2026-09-09-0d31508599a7814a.patch");
/// File name the patch is written to inside the rust repo, to be applied from there
static RUST_PATCH_FILE: &str = "megaton.patch";

pub fn inspect() {
    let rust_env = RustEnvPartial::resolve();
    match rust_env.host {
        None => {
            cu::print!("host rustc: <not installed>")
        }
        Some(env) => {
            cu::print!("host rustc: {}", env.env.release)
        }
    }
    match rust_env.megaton {
        None => {
            cu::print!("megaton rustc: <not installed>")
        }
        Some(env) => {
            cu::print!("megaton rustc: {}", env.release);
            cu::print!("  commit_hash: {}", env.commit_hash);
            cu::print!("  commit_date: {}", env.commit_date);
            cu::print!("  llvm_version: {}", env.llvm_version);
            cu::print!("  targets: {}", env.targets.join(", "));
        }
    }
}

#[cu::context("failed to install megaton rust toolchain")]
pub fn install(home: &HomeEnv, keep: bool, mut clean: bool) -> cu::Result<()> {
    let rust_env = RustEnvPartial::resolve();
    let host_env = cu::check!(rust_env.host, "cannot find rust toolchain, which is required to install megaton's custom rust toolchain; please install rustup")?;

    match rust_env.megaton {
        None => {
            cu::debug!("no toolchain installation found");
        }
        Some(env) => {
            if env.commit_hash == BLESSED_COMMIT {
                cu::warn!(
                "found existing toolchain installation that matched blessed commit hash"
            );
                cu::hint!(
                "if you want to reinstall it, remove it first with 'megaton toolchain remove'."
            );
                return Ok(());
            }
            // older toolchain installed, proceed with reinstallment
            cu::info!("found older toolchain installation with commit hash: {}", env.commit_hash);
            remove(home)?;
        }
    }

    let build_dep = rust_env.build_dep.resolve_full()?;
    let host_triple = &host_env.env.host;

    cu::info!("building rust toolchain for host triple: {host_triple}");
    let rust_path = home.rust_source_path();
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
        clone_rust_source( &build_dep.git, &rust_path)?;
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

    let install_location = home.rust_install_path();
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

    let install_location = home.rust_install_path();
    let toolchain_link_result = host_env.rustup
        .command()
        .add(cu::args![
            "toolchain",
            "link",
            MEGATON_TOOLCHAIN_NAME,
            install_location
        ])
        .all_null()
        .wait_nz();
    cu::check!(toolchain_link_result, "failed to link built toolchain")?;

    let rust_env = cu::check!(
        RustEnv::resolve(),
        "failed to get toolchain, installation might have failed."
    )?;
    let rust_megaton_env = cu::check!(
        rust_env.megaton,
        "failed to get megaton toolchain, installation might have failed."
    )?;

    if rust_megaton_env.commit_hash == BLESSED_COMMIT {
        cu::info!("verified installed toolchain has the blessed commit hash");
    } else {
        cu::warn!("the installed toolchain does not have the blessed commit hash");
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

#[cu::context("failed to uninstall megaton rust toolchain")]
pub fn remove(home: &HomeEnv) -> cu::Result<()> {
    let rust_env = RustEnvPartial::resolve();
    if rust_env.megaton.is_none() {
        cu::info!("megaton rust toolchain is not installed");
    }
    let host_env = cu::check!(rust_env.host, "cannot find rust toolchain, which is required to install megaton's custom rust toolchain; please install rustup")?;
    remove_internal(home, &host_env)
}

fn remove_internal(home: &HomeEnv, host_env: &RustHostEnv) -> cu::Result<()> {
    host_env.rustup
        .command()
        .args(["toolchain", "uninstall", MEGATON_TOOLCHAIN_NAME])
        .name("rustup")
        .stderr(cu::lv::P)
        .stdio_null()
        .wait_nz()?;

    let rust_env = RustEnvPartial::resolve();
    if rust_env.megaton.is_some() {
        cu::bail!(
        "failed to uninstall toolchain. Please run 'rustup toolchain uninstall {MEGATON_TOOLCHAIN_NAME}' to uninstall it manually, then try again."
    );
    }

    let install_path = home.rust_install_path();
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

#[cu::context("failed to clean megaton rust toolchain")]
pub fn clean(home: &HomeEnv) -> cu::Result<()> {
    let rust_path = home.rust_source_path();
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

fn clone_rust_source(git: &Path, path: &Path) -> cu::Result<()> {
    cu::check!(
        cu::fs::make_dir_empty(path),
        "fail to clean rust source directory"
    )?;
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
