use buildcommon::prelude::*;

use std::path::{Path, PathBuf};

use buildcommon::env::Env;
use buildcommon::print::{self, Progress};
use buildcommon::system::Command;
use clap::Args;
use derive_more::derive::Deref;

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
        git_pull(&env)?;
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
        cargo_build(&env, bin_name)?;
    }

    let target_exe = replace_executable(&env, &target_release_path, bin_name).map_err(|err| {
        errorln!("Failed", "Error when replacing executable");
        err
    })?;

    let shim_path = create_shims(&env, &target_exe).map_err(|err| {
        errorln!("Failed", "Error when creating shim script");
        err
    })?;

    if build_skipped {
        hintln!("WARNING", "");
        hintln!("WARNING", "Installation not completed yet!");
        hintln!(
            "WARNING",
            "To ensure installation is successful, follow the steps below"
        );
    } else {
        infoln!("Done", "");
        infoln!("Done", "Installation successful");
    }

    // no need to show warning if PATH is already set up
    if build_skipped
        || !which::which("megaton")
            .map(|path| path == shim_path)
            .unwrap_or_default()
    {
    if build_skipped {
        hintln!("WARNING", "");
        } else {
        infoln!("Done", "");
        }

        let bin_path = env.megaton_home.join("bin");
        
        hintln!(
            "Next",
            "Make sure scripts from '{}' is callable from your shell",
            bin_path.display()
        );
        hintln!("Next", "You can do that by adding the directory to PATH, or");
        hintln!("Next", "  by creating symlinks to the scripts.");
    }

    if build_skipped {
        hintln!("Then", "-----");
        hintln!(
            "Then",
            "Run `megaton install` to properly build the build tool and install it again"
        );
        let err = report!(Error::NeedRerun)
        .attach_printable("follow the steps above to make `megaton` callable from your shell, then rerun the installation");
        return Err(err);
    }

    Ok(())
}

error_context!(GitPull, |r| -> Error {
    r.change_context(Error::InstallUpdate)
});
fn git_pull(env: &Env) -> ResultIn<(), GitPull> {
    infoln!("Updating", "{}", env.megaton_home.display());
    let mut child = Command::new(&env.git)
        .args(args![
            "-C",
            &env.megaton_home,
            "-c",
            print::git_color_flag(),
            "pull",
            "--ff-only"
        ])
        .piped()
        .spawn()?;

    let handle = child
        .take_stderr()
        .map(|stderr| std::thread::spawn(move || Progress::new("Pull", stderr).dump()));

    let child = child.wait()?;
    if let Some(handle) = handle {
        let _ = handle.join();
    }
    let result = child.check();
    if result.is_err() {
        errorln!("Failed", "Update megaton repository");
        hintln!(
            "Consider",
            "Ensure the megaton repository is in a clean state"
        );
    }
    result?;

    Ok(())
}

error_context!(CargoBuild, |r| -> Error {
    r.change_context(Error::InstallCargoBuild)
});
fn cargo_build(env: &Env, bin_name: &str) -> ResultIn<(), CargoBuild> {
    infoln!("Building", "{}", bin_name);

    Command::new(&env.cargo)
        .args(args![
            "build",
            "--release",
            "--bin",
            env!("CARGO_BIN_NAME"),
            "--color",
            print::color_flag()
        ])
        .current_dir(&env.megaton_home)
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

error_context!(ReplaceExe, |r| -> Error {
    r.change_context(Error::ReplaceExe)
});
/// Replace the build tool executable, return path to the new one
fn replace_executable(env: &Env, release: &Path, bin_name: &str) -> ResultIn<PathBuf, ReplaceExe> {
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
        system::copy_file(source_exe, &target_exe)?;
        Ok(target_exe)
    } else {
        infoln!("Replacing", "current executable with {}", bin_name);
        self_replace::self_replace(release.join(bin_name))?;
        Ok(target_exe)
    }
}

error_context!(CreateShim, |r| -> Error {
    r.change_context(Error::CreateShim)
});
/// Create platform-specific shim script
fn create_shims(env: &Env, megaton_buildtool: &Path ) -> ResultIn<PathBuf, CreateShim> {
    let bin_path = env.megaton_home.join("bin");
    // canonicalize just to make sure
    let megaton_buildtool = megaton_buildtool.to_abs()?;
    let megaton_home = env.megaton_home.to_abs()?;

    if cfg!(windows) {

        let content = format!(
            "@echo off\r\n\"{}\" -H \"{}\"%*\r\n",
            megaton_buildtool.display(),
            megaton_home.display()
        );
        let shim_path = bin_path.into_joined("megaton.cmd");
        system::write_file(&shim_path, content)?;
        infoln!("Created", "shim script at '{}'", shim_path.display());


        Ok(shim_path)
    } else {
        let content = format!(
            "#!/usr/bin/bash\nexec \"{}\" -H \"{}\" \"$@\"",
            megaton_buildtool.display(),
            megaton_home.display()
        );
        let shim_path = bin_path.into_joined("megaton");
        system::write_file(&shim_path, content)?;

        #[cfg(not(windows))]
        {
            // set executable bit
            use std::os::unix::fs::PermissionsExt;
            let mut perms = shim_path
                .metadata()
                .attach_printable("while setting executable permission")?
                .permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&shim_path, perms)
                .attach_printable("while setting executable permission")?;
        }

        infoln!("Created", "shim script at '{}'", shim_path.display());
        Ok(shim_path)
    }
}
