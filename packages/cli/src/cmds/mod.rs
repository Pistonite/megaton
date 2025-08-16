mod cmd_toolchain;
use cmd_toolchain::*;

use cu::pre::*;

use crate::env;

// #[doc = concat!("Megaton Build Tool CLI (", env!("MEGATON_COMMIT"), ")")]
/// Megaton Build Tool CLI
///
/// Hello
#[derive(clap::Parser, Clone)]
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
    Version(cu::cli::Flags),
    Toolchain{
        #[clap(subcommand)]
        cmd: CmdToolchain
    }
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
    if cmd.version || matches!(cmd.sub, CmdMegatonSub::Version(_)){
        print_version();
        return Ok(())
    }
    unsafe { env::init_env()? };
    match cmd.sub {
        CmdMegatonSub::Version(_) => unreachable!(),
        CmdMegatonSub::Toolchain { cmd } => cmd.run()
    }
}

fn print_version() {
    cu::init_print_options(cu::lv::Color::Auto, cu::lv::Print::QuietQuiet, None);
    println!("megaton v{} ({})", env!("CARGO_PKG_VERSION"), &env::commit()[0..8]);
}
