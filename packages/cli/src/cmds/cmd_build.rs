// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use cu::pre::*;

use crate::{buildsys::{self, BuildArgs}, env};

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
    pub fn run(self) -> cu::Result<()> {
        env::init()?;
        cu::co::run(async move { buildsys::run(self.args).await })
    }
}
