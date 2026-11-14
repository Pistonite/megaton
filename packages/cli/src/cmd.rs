// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

use cu::pre::*;

use megaton_buildtool::cmd::Cmd as CmdBuild;
use megaton_config::cmd::Cmd as CmdConfigDump;
use megaton_synctool::cmd::Cmd as CmdSync;
use megaton_toolchain::cmd::Cmd as CmdToolchain;

use crate::version::CmdVersion;

static LOGO: &str = r#"
 __    __ ______ ______ ______ ______ ______ __   __  
/\ "-./  \\  ___\\  ___\\  __ \\__  _\\  __ \\ "-.\ \ 
\ \ \-./\ \\  __\ \ \__ \\  __ \_/\ \/ \ \/\ \\ \-.  \
 \ \_\ \ \_\\_____\\_____\\_\ \_\\ \_\\ \_____\\_\\"\_\
  \/_/  \/_//_____//_____//_/\/_/ \/_/ \/_____//_/ \/_/"#;

/// Megaton CLI (https://github.com/Pistonite/megaton)
#[derive(clap::Parser, AsRef)]
#[clap(before_help(LOGO))]
pub struct Cmd {
    /// Specify the config path or project directory
    #[clap(short = 'C', long)]
    config: Option<String>,

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

        let config_path = self.config.as_deref();
        match command {
            CmdSubcommand::Build(cmd) => cmd.run(config_path)?,
            CmdSubcommand::Sync(cmd) => cmd.run(self.config)?,
            CmdSubcommand::Toolchain(cmd) => cmd.run()?,
            CmdSubcommand::ConfigDump(cmd) => cmd.run(config_path)?,
            CmdSubcommand::Version(_) => {}
        }

        Ok(())
    }
}

#[derive(clap::Subcommand)]
pub enum CmdSubcommand {
    /// Build the modding project
    Build(CmdBuild),
    /// Sync files from/to the console file system
    Sync(CmdSync),
    /// Manage the custom Rust toolchain and other tools
    Toolchain(CmdToolchain),
    /// Dump resolved config or toolchain information
    ConfigDump(CmdConfigDump),
    /// Print the version. -v to show toolchain information. -q to only print the version number
    Version(CmdVersion),
}

impl AsRef<cu::cli::Flags> for CmdSubcommand {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            CmdSubcommand::Build(cmd) => cmd.as_ref(),
            CmdSubcommand::Sync(cmd) => cmd.as_ref(),
            CmdSubcommand::Toolchain(cmd) => cmd.as_ref(),
            CmdSubcommand::ConfigDump(cmd) => cmd.as_ref(),
            CmdSubcommand::Version(cmd) => cmd.as_ref(),
        }
    }
}
