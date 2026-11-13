// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use cu::pre::*;

use megaton_config::toolchain::HomeEnv;

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
            println!(
                "megaton v{} ({})",
                env!("CARGO_PKG_VERSION"),
                &COMMIT[0..8]
            );
            return Ok(());
        }
        // verbose mode, run env init which prints debugging info
        // for the environment
        match HomeEnv::resolve() {
            Err(e) => {
                cu::warn!("error resolving home: {e:?}");
            }
        };
        ToolchainEnvPartial::re
        cu::hint!("MEGATON_HOME={}", home_env.path.display());
        if let Err(e) = env::init() {
        };
        cu::hint!("--- toolchain ---");
        if let Err(e) = megaton_toolchain_build::cmd::check() {
            cu::warn!("error while checking toolchain: {e:?}");
        }
        // then print the (long) version info
        cu::hint!("--- build tool ---");
        cu::print!("commit {COMMIT}");
        cu::print!("version {}", env!("CARGO_PKG_VERSION"));
        Ok(())
    }
}
