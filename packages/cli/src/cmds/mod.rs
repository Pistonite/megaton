mod cmd_toolchain;
use cmd_toolchain::*;

use cu::pre::*;

#[derive(clap::Parser, Clone)]
pub struct CmdMegaton {
    #[clap(subcommand)]
    sub: CmdMegatonSub
}

impl AsRef<cu::cli::Flags> for CmdMegaton {
    fn as_ref(&self) -> &cu::cli::Flags {
        self.sub.as_ref()
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum CmdMegatonSub {
    Toolchain{
        #[clap(subcommand)]
        cmd: CmdToolchain
    }
}

impl AsRef<cu::cli::Flags> for CmdMegatonSub {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            CmdMegatonSub::Toolchain { cmd } => cmd.as_ref(),
        }
    }
}

/// Main entry point to running the `megaton` command
pub fn main(cmd: CmdMegaton) -> cu::Result<()> {
    match cmd.sub {
        CmdMegatonSub::Toolchain { cmd } => cmd.run()
    }
}
