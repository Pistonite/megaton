// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

use cu::pre::*;

#[derive(clap::Parser, AsRef)]
struct CmdWrapper {
    /// Specify the config path or project directory
    #[clap(short = 'C', long)]
    config: Option<String>,

    #[clap(flatten)]
    #[as_ref(cu::cli::Flags)]
    cmd: megaton_config::cmd::Cmd,
}
#[cu::cli]
fn main(args: CmdWrapper) -> cu::Result<()> {
    args.cmd.run(args.config.as_deref())
}
