use std::path::{Path, PathBuf};

use buildcommon::env::Env;
use buildcommon::print::{self, Progress};
use buildcommon::system::{self, Command, PathExt};
use buildcommon::{args, errorln, hintln, infoln};
use clap::Args;
use derive_more::derive::Deref;
use error_stack::{report, Result, ResultExt};

use crate::cli::{CommonOptions, TopLevelOptions};
use crate::error::Error;

/// CLI Options for the install command
#[derive(Debug, Clone, PartialEq, Args, Deref)]
pub struct Options {
    /// Pull latest version of the megaton repo with git
    #[clap(short, long)]
    pub update: bool,

    /// Common options
    #[deref]
    #[clap(flatten)]
    pub options: CommonOptions,
}

pub fn run(top: &TopLevelOptions, options: &Options) -> Result<(), Error> {
    let result = run_internal(top, options);
    if result.is_err() {
        if options.update { 
            errorln!("Failed", "Update unsuccessful");
            hintln!("Consider", "Perform a clean installation");
        } else {
            errorln!("Failed", "Install unsuccessful");
        }
    }

    result
}

fn run_internal(top: &TopLevelOptions, options: &Options) -> Result<(), Error> {
    let env = Env::load(top.home.as_deref()).change_context(Error::Install)?;

    if options.update {
        infoln!("Updating", "{}", env.megaton_home.display());
        let mut child = Command::new(&env.git)
            .args(args![
                "-C", &env.megaton_home, 
                "-c", print::git_color_flag(),
                "pull", "--ff-only"
            ])
            .piped()
            .spawn().change_context(Error::InstallUpdate)?;

        let handle = child.take_stderr().map(|stderr| {
            std::thread::spawn(move || {
                Progress::new("Pull", stderr).dump()
            })
        });

        let result = child.wait().change_context(Error::InstallUpdate)?;
        if let Some(handle) = handle {
            let _ = handle.join();
        }
        if !result.is_success() {
            errorln!("Failed", "Git pull failed with status: {}", result.status);
            hintln!("Consider", "Ensure the megaton repository is in a clean state");
            let err = report!(Error::InstallUpdate)
            .attach_printable(format!("git pull failed with status: {}", result.status));
            return Err(err);
        }
        infoln!("Updated", "{}", env.megaton_home.display());

    }

    let bin_name = if cfg!(windows) {
        concat!(env!("CARGO_BIN_NAME"), ".exe")
    } else {
        env!("CARGO_BIN_NAME")
    };

    let target_release_path = env.megaton_home.join("target").into_joined("release");

    // skip build if running as the release binary
    // such as cargo run --release
    // since running cargo build will attempt to override the running binary
    // which might fail
    let mut build_skipped = false;
    if let Ok(exe_path) = std::env::current_exe() {
        let release_exe = target_release_path.join(bin_name);
        if let Ok(exe_path) = exe_path.to_abs() {
            if exe_path == release_exe {
                build_skipped = true;
                hintln!("Skipping", "Build because running as {}", bin_name);
            }
        }
    }

    if !build_skipped {
        infoln!("Building", "{}", bin_name);

        let child = Command::new(&env.cargo)
            .args(args![
                "build", "--release", "--bin", env!("CARGO_BIN_NAME"),
                "--color", print::color_flag()
            ])
                .current_dir(&env.megaton_home)
            .spawn().change_context(Error::InstallCargoBuild)?
        .wait().change_context(Error::InstallCargoBuild)?;

        if !child.is_success() {
            errorln!("Failed", "Cargo build failed with status: {}", child.status);
            let err = report!(Error::InstallCargoBuild)
            .attach_printable(format!("cargo build failed with status: {}", child.status));
            return Err(err);
        }

        infoln!("Built", "{}", bin_name);
    }

    let target_exe = match replace_executable(&env, &target_release_path, bin_name) {
        Ok(target_exe) => target_exe,
        Err(err) => {
            errorln!("Failed", "Error when replaceing executable");
            return Err(err);
        }
    };

    let shim_path = match create_shim(&env, &target_exe) {
        Ok(shim_path) => shim_path,
        Err(err) => {
            errorln!("Failed", "Error when creating shim script");
            return Err(err);
        }
    };

    if build_skipped {
        hintln!("WARNING", "");
        hintln!("WARNING", "Installation not completed yet!");
        hintln!("WARNING", "");
        hintln!("WARNING", "Build was skipped.");
        hintln!("WARNING", "To ensure installation is successful, follow the steps below");
        hintln!("WARNING", "");
    } else {
        infoln!("Done", "Installation successful");
    }

    hintln!("Next", "Make sure '{}' is callable from your shell by:", shim_path.display());
    hintln!("Next", "  1) Adding the directory to `PATH`, or");
    hintln!("Next", "  2) Creating a symlink in a directory that is already in `PATH`, or");
    hintln!("Next", "  3) Using an alias in your shell configuration");
    hintln!("Next", "  4) Copying the shim script to a directory that is already in `PATH`");

    if build_skipped {
        hintln!("Then", "-----");
        hintln!("Then", "Run `megaton install` to properly build the build tool and install it again");
        let err = report!(Error::NeedRerun)
        .attach_printable("follow the steps above to make `megaton` callable from your shell, then rerun the installation");
        return Err(err);
    }

    Ok(())
}

/// Replace the build tool executable, return path to the new one
fn replace_executable(env: &Env, release: &Path, bin_name: &str) -> Result<PathBuf, Error> {
    let target_exe = if cfg!(windows) {
        concat!(env!("CARGO_BIN_NAME"), "-install.exe")
    } else {
        concat!(env!("CARGO_BIN_NAME"), "-install")
    };
    let target_exe = release.join(target_exe);

    let mut copy = true;
    if target_exe.exists() {
        if let Ok(current_exe) = std::env::current_exe() {
            if current_exe == target_exe {
                // if running the install binary, replace it instead
                copy = false;
            }
        }
    }

    if copy {
        let source_exe = release.join(bin_name);
        let target_exe_display = target_exe.rebase(&env.megaton_home);
        infoln!("Creating", "{}", target_exe_display.display());
        system::copy_file(source_exe, &target_exe).change_context(Error::ReplaceExe)?;
        infoln!("Created", "{}", target_exe_display.display());
        Ok(target_exe)
    } else {
        infoln!("Replacing", "current executable with {}", bin_name);
        self_replace::self_replace(release.join(bin_name)).change_context(Error::ReplaceExe)?;
        infoln!("Replaced", "current executable");
        Ok(std::env::current_exe().change_context(Error::ReplaceExe)?)
    }
}

/// Create platform-specific shim script
fn create_shim(env: &Env, target_exe: &Path) -> Result<PathBuf, Error> {
    let bin_path = env.megaton_home.join("bin");
    // canonicalize just to make sure
    let target_exe = target_exe.to_abs().change_context(Error::CreateShim)?;
let megaton_home = env.megaton_home.to_abs().change_context(Error::CreateShim)?;
    if cfg!(windows) {
        let content = format!("@echo off\r\n\"{}\" -H \"{}\"%*\r\n", target_exe.display(), megaton_home.display());
        let shim_path = bin_path.into_joined("megaton.cmd");
        system::write_file(&shim_path, content).change_context(Error::CreateShim)?;
        infoln!("Created", "shim script at '{}'", shim_path.display());
        Ok(shim_path)
    } else {
        let content = format!("#!/usr/bin/bash\nexec \"{}\" -H \"{}\" \"$@\"", target_exe.display(), megaton_home.display());
        let shim_path = bin_path.into_joined("megaton");
        system::write_file(&shim_path, content).change_context(Error::CreateShim)?;
        set_executable(&shim_path)?;
        infoln!("Created", "shim script at '{}'", shim_path.display());
        Ok(shim_path)
    }
}

#[cfg(not(windows))]
#[inline]
fn set_executable(path: &Path) -> Result<(), Error> {
    // set executable bit
    use std::os::unix::fs::PermissionsExt;
    let mut perms = path.metadata().change_context(Error::CreateShim)
        .attach_printable("while setting executable permission")
        ?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(path, perms).change_context(Error::CreateShim)
        .attach_printable("while setting executable permission")?;

    Ok(())
}

#[cfg(windows)]
#[inline]
fn set_executable(path: &Path) -> Result<(), Error> {
    Ok(())
}
