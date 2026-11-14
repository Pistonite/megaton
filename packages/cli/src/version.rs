// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

use cu::pre::*;

use megaton_config::toolchain::ToolchainEnv;

static COMMIT: &str = env!("MEGATON_COMMIT");

/// The version subcommand
#[derive(clap::Parser, AsRef)]
pub struct CmdVersion {
    #[clap(flatten)]
    #[as_ref]
    flags: cu::cli::Flags,
}

impl CmdVersion {
    pub fn run() -> cu::Result<()> {
        if !cu::lv::I.enabled() {
            println!("{}", env!("CARGO_PKG_VERSION"),);
            return Ok(());
        }
        cu::lv::disable_print_time();
        if !cu::lv::D.enabled() {
            println!("megaton v{} ({})", env!("CARGO_PKG_VERSION"), &COMMIT[0..8]);
            return Ok(());
        }
        cu::print!("megaton version {} ({COMMIT})", env!("CARGO_PKG_VERSION"));
        // verbose mode, run env init which prints debugging info
        // for the environment
        match ToolchainEnv::resolve_partial() {
            Err(e) => {
                cu::warn!("error resolving toolchain: {e:?}");
            }
            Ok(toolchain) => {
                cu::debug!("MEGATON_HOME={}", toolchain.home.path.display());
                match &toolchain.devkita64 {
                    None => {
                        cu::debug!("devkita64: <not resolved>");
                    }
                    Some(x) => {
                        cu::debug!("devkita64: gcc {}", x.version);
                    }
                }
                if toolchain.cxxbridge.is_installed() {
                    cu::debug!("cxxbridge: {}", toolchain.cxxbridge.version);
                } else {
                    cu::debug!("cxxbridge: <not installed>");
                }
                let rust = toolchain.rust;
                match &rust.host {
                    None => {
                        cu::debug!("rust (host): <not resolved>");
                    }
                    Some(x) => {
                        cu::debug!("rust (host): {} ({})", x.env.release, x.env.commit_hash);
                    }
                }
                match &rust.megaton {
                    None => {
                        cu::debug!("rust (megaton): <not installed>");
                    }
                    Some(x) => {
                        cu::debug!("rust (megaton): {} ({})", x.release, x.commit_hash);
                    }
                }
                fn get_installed_str(installed: bool) -> &'static str {
                    if installed {
                        "<installed>"
                    } else {
                        "<not installed>"
                    }
                }
                cu::debug!(
                    "build_dep/git: {}",
                    get_installed_str(rust.build_dep.git.is_some())
                );
                cu::debug!(
                    "build_dep/ninja: {}",
                    get_installed_str(rust.build_dep.ninja.is_some())
                );
                cu::debug!(
                    "build_dep/cmake: {}",
                    get_installed_str(rust.build_dep.cmake.is_some())
                );
            }
        }
        Ok(())
    }
}
