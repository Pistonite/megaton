
use cu::pre::*;

use crate::env;

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
            println!(
                "{}",
                env!("CARGO_PKG_VERSION"),
            );
            return Ok(());
        }
        cu::lv::disable_print_time();
        if !cu::lv::D.enabled() {
            println!(
                "megaton v{} ({})",
                env!("CARGO_PKG_VERSION"),
                &env::commit()[0..8]
            );
            return Ok(());
        }
        // verbose mode, run env init which prints debugging info
        // for the environment
        cu::hint!("--- environment ---");
        if let Err(e) = env::init() {
            cu::warn!("error while initializing environment: {e:?}");
        };
        cu::hint!("--- toolchain ---");
        if let Err(e) = megaton_toolchain_build::cmd::check() {
            cu::warn!("error while checking toolchain: {e:?}");
        }
        // then print the (long) version info
        cu::hint!("--- build tool ---");
        cu::print!("commit {}", env::commit());
        cu::print!("version {}", env!("CARGO_PKG_VERSION"));
        Ok(())
    }
}
