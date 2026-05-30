
use cu::pre::*;

use crate::buildsys::{self, BuildArgs};

/// The `build` subcommand
#[derive(Debug, AsRef, clap::Parser)]
pub struct CmdBuild {
    #[clap(flatten)]
    args: BuildArgs,

    #[clap(flatten)]
    #[as_ref]
    common: cu::cli::Flags,
}

impl CmdBuild {
    pub async fn run(self) -> cu::Result<()> {
        buildsys::run(self.args).await
    }
}
