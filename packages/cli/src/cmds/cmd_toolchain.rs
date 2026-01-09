// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::env;

/// The Rust toolchain name to be installed/linked
static TOOLCHAIN_NAME: &str = "megaton";
/// The Rust compiler repo
static RUST_REPO: &str = "https://github.com/rust-lang/rust";
/// The "blessed" commit hash to use (i.e. tested and will work)
static RUST_COMMIT: &str = "caadc8df3519f1c92ef59ea816eb628345d9f52a";

/// Manage the custom `megaton` Rust toolchain
#[derive(Debug, Clone, clap::Subcommand)]
pub enum CmdToolchain {
    /// Check the installation status of the toolchain
    Check(cu::cli::Flags),
    /// Build and install the toolchain
    Install {
        /// Keep the rustc/llvm build output. This may consume a lot of disk space,
        /// but makes it faster when debugging the toolchain
        #[clap(short, long)]
        keep: bool,

        /// Clean build rustc/llvm
        #[clap(short, long)]
        clean: bool,

        #[clap(flatten)]
        common: cu::cli::Flags,
    },
    /// Uninstall the toolchain
    Remove(cu::cli::Flags),
    /// Remove artifacts from building Rust compiler
    Clean(cu::cli::Flags),
}

impl AsRef<cu::cli::Flags> for CmdToolchain {
    fn as_ref(&self) -> &cu::cli::Flags {
        match &self {
            Self::Check(args) => args,
            Self::Install { common, .. } => common,
            Self::Remove(args) => args,
            Self::Clean(args) => args,
        }
    }
}

impl CmdToolchain {
    pub fn run(self) -> cu::Result<()> {
        match self {
            Self::Install { keep, clean, .. } => install(keep, clean),
            Self::Check(_) => check(),
            Self::Remove(_) => remove(),
            Self::Clean(_) => clean(),
        }
    }
}
fn install(keep: bool, clean: bool) -> cu::Result<()> {
    cu::which("rustup")
        .context("rustup is required to manage Rust toolchains. Please install Rust")?;
    cu::which("rustc")
        .context("rustc is required to manage Rust toolchains. Please install Rust")?;

    match check_toolchain(false) {
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
                            cu::bailfyi!("aborted by user");
                        }
                        Ok(true) => {}
                    }
                }
                Some(hash) => {
                    if hash == RUST_COMMIT {
                        cu::warn!(
                            "found existing toolchain installation that matched blessed commit hash"
                        );
                        cu::hint!(
                            "if you want to reinstall it, remove it first with 'megaton toolchain remove'."
                        );
                        cu::bailfyi!("toolchain already installed");
                    }
                    // older toolchain installed, proceed with reinstallment
                    cu::info!("found older toolchain installation with commit hash: {hash}");
                    remove_toolchain_internal()?;
                }
            }
        }
    }

    cu::which("ninja").context("ninja is required to build llvm")?;
    cu::which("cmake").context("cmake is required to build llvm")?;

    let host_triple = get_rustc_host_triple()?;
    cu::info!("building megaton toolchain for host triple: {host_triple}");
    let rust_path = rust_source_location();
    if clean {
        cu::warn!("--clean is specified, performing full re-checkout");
        clone_rust_source(&rust_path)?;
        checkout_blessed_commit(&rust_path)?;
    } else {
        // try to get the current commit hash, will succeed if we have a valid repo
        match try_get_rust_source_commit(&rust_path) {
            Ok(hash) => {
                if hash == RUST_COMMIT {
                    cu::info!("blessed commit is already checkout, skipping update");
                } else {
                    cu::info!(
                        "current commit is not the blessed commit, checking out the blessed commit"
                    );
                    checkout_blessed_commit(&rust_path)?;
                }
            }
            Err(e) => {
                cu::debug!("cannot get current commit: {e}");
                clone_rust_source(&rust_path)?;
                checkout_blessed_commit(&rust_path)?;
            }
        }
    }

    // verify the blessed commit is checked out
    let actual_commit = try_get_rust_source_commit(&rust_path)
        .context("failed to verify the blessed commit is checked out")?;
    if actual_commit != RUST_COMMIT {
        cu::bail!("failed to checkout the blessed commit.");
    }

    let mut bootstrap_toml = String::new();
    // using the compiler profile, since it usually builds the fastest (compared to other)
    bootstrap_toml += "profile = 'compiler'\n";
    let change_id = get_change_id(&rust_path)?;
    cu::info!("change-id: {change_id}");
    bootstrap_toml += &format!("change-id = {change_id}\n");

    // llvm configs
    // even though it's faster to download ci-llvm, in my experience
    // it will just fail
    bootstrap_toml += "llvm.download-ci-llvm = false\n";
    if host_triple.starts_with("x86_64-") {
        bootstrap_toml += "llvm.targets = 'AArch64;X86'\n";
    } else if host_triple.starts_with("aarch64-") {
        bootstrap_toml += "llvm.targets = 'AArch64'\n";
    } else {
        cu::warn!("using default llvm targets since the host is neither x86_64 or aarch64");
    };

    // build configs
    bootstrap_toml += "build.compiler-docs = false\n";
    bootstrap_toml += "build.extended = false\n";
    // stage 2 compiler will be newer, but doubly slow to build
    // (basically fresh built from stage 1 compiler)
    bootstrap_toml += "build.build-stage = 1\n";
    bootstrap_toml += &format!("build.host = ['{host_triple}']\n");
    // TODO: building nintendo switch target just for testing, when hermit is mature, can remove
    // the other, to make it build faster
    bootstrap_toml += &format!(
        "build.target = ['{host_triple}', 'aarch64-unknown-hermit', 'aarch64-nintendo-switch-freestanding']\n"
    );

    // install configs
    let install_location = toolchain_install_location();
    cu::fs::make_dir_empty(&install_location)
        .context("failed to create toolchain install location")?;
    let install_location = install_location
        .normalize_exists()
        .context("failed to create toolchain install location")?;
    bootstrap_toml += &format!(
        "install.prefix = '{}'\n",
        install_location
            .as_utf8()
            .context("install location must be UTF-8")?
    );
    bootstrap_toml += &format!(
        "install.sysconfdir = '{}'\n",
        install_location
            .join("etc")
            .into_utf8()
            .context("install location must be UTF-8")?
    );

    // rust build configs
    bootstrap_toml += "rust.debug-logging = false\n";
    bootstrap_toml += "rust.debug-assertions = false\n";
    bootstrap_toml += "rust.debuginfo-level = 0\n";
    bootstrap_toml += "rust.backtrace-on-ice = false\n";
    bootstrap_toml += "rust.frame-pointers = false\n";
    bootstrap_toml += "rust.download-rustc = false\n";
    bootstrap_toml += "rust.incremental = false\n";
    // reducing this will make the built compiler faster,
    // but will make building the compiler slow, which is not what we want
    // (since the mod is usually small and pretty fast to build anyway)
    bootstrap_toml += "rust.codegen-units = 16\n";
    // https://github.com/rust-lang/rust/blob/master/bootstrap.example.toml
    // anything other than 1 "occasionally have bugs"
    bootstrap_toml += "rust.codegen-units-std = 1\n";
    // we need the hash to check when rebuild is needed
    bootstrap_toml += "rust.omit-git-hash = false\n";
    if host_triple == "x86_64-unknown-linux-gnu" {
        bootstrap_toml += "rust.lto = 'thin'\n";
    }

    cu::fs::write(rust_path.join("bootstrap.toml"), bootstrap_toml)
        .context("failed to write bootstrap config")?;

    cu::info!("building and installing rust");
    cu::hint!(" - this may take a while, please be patient.");
    {
        let debug_log = cu::log_enabled(cu::lv::D);
        let command = cu::bin::resolve("rust-x", rust_path.join("x"))?
            .command()
            .current_dir(&rust_path)
            .add(cu::color_flag())
            .args(["--stage", "1", "install", "compiler/rustc", "library/std"])
            .stdin_null();
        let code = if debug_log {
            command.stdoe(cu::pio::inherit()).wait()?
        } else {
            let (child, _, _) = command.stdoe(cu::pio::spinner("")).spawn()?;
            child.wait()?
        };

        if !code.success() {
            if !debug_log {
                cu::hint!("enable verbose output -v to see the output from rust/x");
            }
            cu::bail!("rust/x failed!");
        }
    }

    cu::which("rustup")?
        .command()
        .add(cu::args![
            "toolchain",
            "link",
            TOOLCHAIN_NAME,
            toolchain_install_location()
        ])
        .all_null()
        .wait_nz()
        .context("failed to link built toolchain")?;

    let toolchain_info = check_toolchain(true)
        .context("failed to get toolchain, installation might have failed.")?;
    let toolchain_info = cu::check!(
        toolchain_info,
        "failed to get toolchain, installation might have failed."
    )?;
    match toolchain_info.commit_hash {
        None => {
            cu::warn!("failed to get commit hash from installed toolchain");
        }
        Some(hash) => {
            if hash == RUST_COMMIT {
                cu::info!("verified installed toolchain has the blessed commit hash");
            } else {
                cu::warn!("the installed toolchain does not have the blessed commit hash");
            }
        }
    }

    if !keep {
        cu::info!("removing build artifacts to free disk space");
        cu::hint!("- use --keep if you want to keep them");
        let _bar = cu::progress_unbounded("removing build artifacts");
        cu::fs::rec_remove(rust_path)?;
    } else {
        cu::hint!("keeping build artifacts since --keep is specified");
    }

    cu::info!("toolchain installed successfully!");

    Ok(())
}

fn check() -> cu::Result<()> {
    cu::info!("checking toolchain status with rustup");
    if check_toolchain(true)?.is_none() {
        cu::warn!("{TOOLCHAIN_NAME} toolchain is not found!");
        cu::hint!("run `megaton toolchain install` to install the toolchain");
    }
    Ok(())
}

fn remove() -> cu::Result<()> {
    match check_toolchain(true) {
        Err(e) => {
            cu::error!("failed to check existing toolchain status: {e}");
        }
        Ok(None) => {
            cu::info!("{TOOLCHAIN_NAME} is not installed.");
            return Ok(());
        }
        Ok(Some(_)) => {}
    }
    cu::info!("uninstalling toolchain");
    remove_toolchain_internal()
}

fn clean() -> cu::Result<()> {
    let rust_path = rust_source_location();
    if rust_path.exists() {
        let _bar = cu::progress_unbounded("removing rust repo");
        if let Err(e) = cu::fs::rec_remove(rust_path) {
            cu::warn!("failed to remove rust repo: {e}");
        }
    } else {
        cu::info!("rust repo does not exist");
    }
    Ok(())
}

fn remove_toolchain_internal() -> cu::Result<()> {
    cu::which("rustup")?
        .command()
        .args(["toolchain", "uninstall", TOOLCHAIN_NAME])
        .name("rustup")
        .stderr(cu::lv::P)
        .stdio_null()
        .wait_nz()?;
    if let Ok(Some(_)) = check_toolchain(false) {
        cu::bailand!(warn!(
            "failed to uninstall toolchain. Please run 'rustup toolchain uninstall {TOOLCHAIN_NAME}' to uninstall it manually, then try again."
        ));
    }

    let install_path = toolchain_install_location();
    cu::debug!(
        "cleaning up toolchain files at '{}'",
        install_path.display()
    );
    cu::fs::make_dir_empty(install_path).context("failed to clean up old toolchain files")?;
    Ok(())
}

fn toolchain_install_location() -> PathBuf {
    env::environment().home().join("rust-toolchain")
}

fn clone_rust_source(path: &Path) -> cu::Result<()> {
    cu::fs::make_dir_empty(path).context("fail to clean rust source directory")?;
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
    git.command()
        .add(cu::args![
            "-C",
            &path,
            "fetch",
            "origin",
            RUST_COMMIT,
            "--progress",
            "--depth",
            "1"
        ])
        .stdoe(cu::pio::spinner("fetching rust source"))
        .stdin_null()
        .spawn()?
        .0
        .wait()?;
    git.command()
        .add(cu::args![
            "-C",
            &path,
            "checkout",
            RUST_COMMIT,
            "--progress"
        ])
        .stdoe(cu::pio::spinner("checking-out rust source"))
        .stdin_null()
        .spawn()?
        .0
        .wait()?;
    Ok(())
}

/// Get the rust source commit of the currently checked out rust repo,
/// if any
fn try_get_rust_source_commit(path: &Path) -> cu::Result<String> {
    if !path.join(".git").exists() {
        cu::bail!("not a git repo");
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

fn get_change_id(path: &Path) -> cu::Result<String> {
    let change_tracker_path = path.join("src/bootstrap/src/utils/change_tracker.rs");
    let source = cu::fs::read_string(change_tracker_path)?;
    let mut change_id = None;
    for line in source.lines().rev() {
        let line = line.trim();
        if let Some(after) = line.strip_prefix("change_id: ") {
            change_id = Some(after.trim_matches(','));
            break;
        }
    }
    let Some(change_id) = change_id else {
        cu::bail!("cannot find change-id from change_tracker.rs");
    };
    Ok(change_id.to_string())
}

fn rust_source_location() -> PathBuf {
    env::environment().home().join("rust")
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

struct ToolchainInfo {
    commit_hash: Option<String>,
}

/// Check the current status of the megaton Rust toolchain
fn check_toolchain(print: bool) -> cu::Result<Option<ToolchainInfo>> {
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
    let output = out.join().flatten().unwrap_or_default();
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
    Ok(Some(ToolchainInfo { commit_hash }))
}
