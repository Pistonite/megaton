// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

mod cmd_toolchain;
use cmd_toolchain::*;

use cu::pre::*;

use crate::env;

static LOGO: &str = r#"
 __    __ ______ ______ ______ ______ ______ __   __  
/\ "-./  \\  ___\\  ___\\  __ \\__  _\\  __ \\ "-.\ \ 
\ \ \-./\ \\  __\ \ \__ \\  __ \_/\ \/ \ \/\ \\ \-.  \
 \ \_\ \ \_\\_____\\_____\\_\ \_\\ \_\\ \_____\\_\\"\_\
  \/_/  \/_//_____//_____//_/\/_/ \/_/ \/_____//_/ \/_/"#;

/// Megaton Build Tool
#[derive(clap::Parser, Clone)]
#[clap(before_help(LOGO))]
pub struct CmdMegaton {
    #[clap(subcommand)]
    sub: CmdMegatonSub,

    /// Print the version
    #[clap(short, long)]
    version: bool,
}

impl AsRef<cu::cli::Flags> for CmdMegaton {
    fn as_ref(&self) -> &cu::cli::Flags {
        self.sub.as_ref()
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum CmdMegatonSub {
    /// Print the version and build information
    Version(cu::cli::Flags),
    Toolchain {
        #[clap(subcommand)]
        cmd: CmdToolchain,
    },
}

impl AsRef<cu::cli::Flags> for CmdMegatonSub {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            Self::Version(x) => x,
            Self::Toolchain { cmd } => cmd.as_ref(),
        }
    }
}

/// Main entry point to running the `megaton` command
pub fn main(cmd: CmdMegaton) -> cu::Result<()> {
    if cmd.version || matches!(cmd.sub, CmdMegatonSub::Version(_)) {
        print_version();
        return Ok(());
    }
    // safety: program is single threaded at this point
    unsafe { env::init_env()? };
    match cmd.sub {
        CmdMegatonSub::Version(_) => unreachable!(),
        CmdMegatonSub::Toolchain { cmd } => cmd.run(),
    }
}

fn print_version() {
    println!(
        "megaton v{} ({})",
        env!("CARGO_PKG_VERSION"),
        &env::commit()[0..8]
    );
    cu::disable_print_time();
}
