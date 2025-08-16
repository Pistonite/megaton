use std::path::PathBuf;

use cu::pre::*;

/// The Rust toolchain name to be installed/linked
static TOOLCHAIN_NAME: &str = "megaton";
/// The Rust compiler repo
static RUST_REPO: &str = "https://github.com/rust-lang/rust";

#[derive(Debug, Clone, clap::Parser)]
struct Args {
    #[clap(subcommand)]
    command: ArgCommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum ArgCommand {
    /// Check the installation status of the toolchain
    Check(cu::cli::Flags),
    /// Build and install the toolchain
    Install(cu::cli::Flags),
    /// Uninstall the toolchain
    Remove(cu::cli::Flags),
    /// Uninstall and remove any build artifacts
    Clean(cu::cli::Flags),
}

impl AsRef<cu::cli::Flags> for Args {
    fn as_ref(&self) -> &cu::cli::Flags {
        match &self.command {
            ArgCommand::Check(args) => args,
            ArgCommand::Install(args) => args,
            ArgCommand::Remove(args) => args,
            ArgCommand::Clean(args) => args,
        }
    }
}

#[cu::cli]
fn main(args: Args) -> cu::Result<()> {
    match args.command {
        ArgCommand::Check(_) => check(),
        ArgCommand::Install(_) => install(),
        ArgCommand::Remove(_) => remove(),
        ArgCommand::Clean(_) => clean(),
    }
}

fn clean() -> cu::Result<()> {
    remove()?;
    {
        let _bar = cu::progress_unbounded("removing rust repo");
        cu::fs::remove_dir(get_rust_clone_path()?)?;
    }
    Ok(())
}

fn check() -> cu::Result<()> {
    let installed = check_toolchain()?;
    if !installed {
        cu::warn!("{TOOLCHAIN_NAME} toolchain is not found!");
        cu::hint!("run `megaton toolchain install` to install the toolchain");
    } else {
        cu::hint!("run `megaton toolchain remove` to remove the toolchain");
    }
    Ok(())
}

fn remove() -> cu::Result<()> {
    if let Ok(false) = check_toolchain() {
        cu::info!("toolchain is not installed, nothing to do.");
        return Ok(());
    }
    cu::info!("uninstalling megaton toolchain");
    cu::which("rustup")?
        .command()
        .args(["toolchain", "uninstall", TOOLCHAIN_NAME])
        .stdout(cu::lv::P)
        .stdie_null()
        .wait_nz()?;
    if check_toolchain()? {
        cu::bailand!(warn!(
            "rustup succeeded, but the toolchain was not uninstalled"
        ));
    }
    Ok(())
}

fn install() -> cu::Result<()> {
    cu::which("rustup").context("rustup is required")?;

    if let Ok(true) = check_toolchain() {
        cu::hint!(
            "{TOOLCHAIN_NAME} toolchain is already installed. If you want to reinstall it, run `megaton toolchain remove` to remove it first."
        );
        return Ok(());
    }

    cu::which("ninja").context("ninja is required to build llvm")?;
    cu::which("cmake").context("cmake is required to build llvm")?;

    let host_triple = get_rustc_host_triple()?;
    let rust_path = get_rust_clone_path()?;

    if rust_path.exists() {
        cu::warn!("rust directory exists, not recloning.");
        cu::hint!("to reclone rust repo, run clean first");
    } else {
        cu::debug!("cloning rust compiler repo");
        cu::fs::remove_dir(&rust_path)?;
        cu::fs::ensure_dir(&rust_path)?;
        cu::which("git")?
            .command()
            .add(cu::args![
                "clone",
                "--depth",
                "1",
                "--progress",
                RUST_REPO,
                &rust_path
            ])
            .stderr(cu::pio::spinner("cloning rust").info())
            .stdio_null()
            .wait_nz()?;
    }

    cu::debug!("reading the change-id");
    let change_id = {
        let change_tracker_path = rust_path.join("src/bootstrap/src/utils/change_tracker.rs");
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
        change_id.to_string()
    };
    cu::debug!("change-id is: {change_id}");

    cu::info!("writing bootstrap.toml");

    let bootstrap_toml = format!(
        r#"
profile = "compiler"
change-id = {change_id}

# might have issues when downloading, just build from source
[llvm]
download-ci-llvm = false 

[build]
build-stage = 1
host = ["{host_triple}"]
target = ["{host_triple}", "aarch64-unknown-hermit", "aarch64-nintendo-switch-freestanding"]
"#
    );
    cu::fs::write(rust_path.join("bootstrap.toml"), bootstrap_toml)?;

    cu::info!("building rust compiler");
    cu::hint!("this will take some time, please be patient");
    cu::bin::resolve("rustc-x", rust_path.join("x"))?
        .command()
        .current_dir(&rust_path)
        .add(cu::color_flag())
        .args(["build", "--stage", "1", "library"])
        .stdoe(cu::pio::spinner(""))
        .stdin_null()
        .wait_nz()?;

    cu::info!("linking toolchain");
    cu::which("rustup")?
        .command()
        .add(cu::args![
            "toolchain",
            "link",
            TOOLCHAIN_NAME,
            rust_path.join("build/host/stage1")
        ])
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;

    if !check_toolchain()? {
        cu::bailand!(error!(
            "toolchain was built succesfully, but installation failed!"
        ));
    }

    Ok(())
}

/// Get rustc host triple by running `rustc -vV`, like `x86_64-unknown-linux-gnu`
fn get_rustc_host_triple() -> cu::Result<String> {
    cu::debug!("finding host triple");
    let (child, output) = cu::which("rustc")?
        .command()
        .arg("-vV")
        .stdout(cu::pio::lines())
        .stdie_null()
        .spawn()?;
    let child = child.wait_guard();
    for line in output {
        let line = line.context("failed to read rustc output")?;
        if let Some(host) = line.strip_prefix("host: ") {
            cu::debug!("host triple is {host}");
            return Ok(host.to_string());
        }
    }
    child.wait_nz()?;
    cu::bail!("failed to get host triple from rustc");
}

/// Check if `megaton` toolchain is installed, and print it's info if it is
fn check_toolchain() -> cu::Result<bool> {
    let (child, output) = cu::which("rustup")?
        .command()
        .args(["toolchain", "list", "-v"])
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let output = output.join()??;
    let mut found_toolchain = false;
    let prefix = format!("{TOOLCHAIN_NAME} ");
    for line in output.lines() {
        let line = line.trim();
        if let Some(path) = line.strip_prefix(&prefix) {
            found_toolchain = true;
            cu::info!("found megaton toolchain at: {path}");
            break;
        }
    }
    if !found_toolchain {
        return Ok(false);
    }

    cu::which("rustc")?
        .command()
        .arg(format!("+{TOOLCHAIN_NAME}"))
        .arg("-vV")
        .stdout(cu::lv::P)
        .stdie_null()
        .wait_nz()?;

    Ok(true)
}

fn get_rust_clone_path() -> cu::Result<PathBuf> {
    let mut p = cu::fs::current_exe()?.parent_abs()?;
    p.push(".megaton/rust");
    Ok(p)
}
