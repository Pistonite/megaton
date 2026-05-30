// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use cu::pre::*;

use crate::cmds::{CmdToolchain, CmdVersion, CmdBuild};

static LOGO: &str = r#"
 __    __ ______ ______ ______ ______ ______ __   __  
/\ "-./  \\  ___\\  ___\\  __ \\__  _\\  __ \\ "-.\ \ 
\ \ \-./\ \\  __\ \ \__ \\  __ \_/\ \/ \ \/\ \\ \-.  \
 \ \_\ \ \_\\_____\\_____\\_\ \_\\ \_\\ \_____\\_\\"\_\
  \/_/  \/_//_____//_____//_/\/_/ \/_/ \/_____//_/ \/_/"#;

/// Megaton Build Tool CLI
#[derive(clap::Parser, AsRef)]
#[clap(before_help(LOGO))]
pub struct Cmd {
    #[clap(subcommand)]
    command: Option<CmdSubcommand>,

    /// Same as the version subcommand
    #[clap(short = 'V', long)]
    version: bool,

    #[clap(flatten)]
    #[as_ref]
    flags: cu::cli::Flags,
}

impl Cmd {
    pub fn preprocess(&mut self) {
        if let Some(command) = &self.command {
            self.flags.merge(command.as_ref());
        }
    }
    pub fn run(self) -> cu::Result<()> {
        if self.version || matches!(self.command, Some(CmdSubcommand::Version(_))) {
            return CmdVersion::run();
        }
        let Some(command) = self.command else {
            cu::cli::print_help::<Self>(false);
            cu::lv::disable_print_time();
            return Ok(());
        };
        match command {
            CmdSubcommand::Build(cmd) => cmd.run()?,
            CmdSubcommand::Toolchain(cmd) => cmd.run()?,
            CmdSubcommand::Version(_) => {},
        }

        Ok(())
    }
}

#[derive(clap::Subcommand)]
pub enum CmdSubcommand {
    /// Build the project into an executable
    Build(CmdBuild),
    /// Manage the custom `megaton` Rust toolchain
    Toolchain(CmdToolchain),
    /// Print the version. -v to show toolchain information. -q to only print the version number
    Version(CmdVersion),
}

impl AsRef<cu::cli::Flags> for CmdSubcommand {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            CmdSubcommand::Build(cmd) => cmd.as_ref(),
            CmdSubcommand::Toolchain(cmd) => cmd.as_ref(),
            CmdSubcommand::Version(cmd) => cmd.as_ref(),
        }
    }
}
