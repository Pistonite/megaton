// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use cu::pre::*;

/// The `toolchain` subcommand
#[derive(Debug, clap::Parser)]
pub struct CmdToolchain {
    #[clap(subcommand)]
    command: CmdToolchainSubcommand,
}
impl CmdToolchain {
    pub fn run(self) -> cu::Result<()> {
        match self.command {
            CmdToolchainSubcommand::Install { keep, clean, .. } => {
                megaton_toolchain_build::cmd::install(keep, clean)
            }
            CmdToolchainSubcommand::Remove(_) => megaton_toolchain_build::cmd::remove(),
            CmdToolchainSubcommand::Clean(_) => megaton_toolchain_build::cmd::clean(),
        }
    }
}
impl AsRef<cu::cli::Flags> for CmdToolchain {
    fn as_ref(&self) -> &cu::cli::Flags {
        match &self.command {
            CmdToolchainSubcommand::Install { common, .. } => common,
            CmdToolchainSubcommand::Remove(args) => args,
            CmdToolchainSubcommand::Clean(args) => args,
        }
    }
}

#[derive(Debug, clap::Subcommand)]
enum CmdToolchainSubcommand {
    /// Build and install the toolchain
    Install {
        /// Keep the rustc/llvm build output. This may consume a lot of disk space,
        /// but makes it faster when debugging the toolchain
        #[clap(short, long)]
        keep: bool,

        /// Clean build rustc/llvm
        #[clap(short, long)]
        clean: bool,

        #[clap(flatten)]
        common: cu::cli::Flags,
    },
    /// Uninstall the toolchain
    Remove(cu::cli::Flags),
    /// Remove artifacts from building Rust compiler
    Clean(cu::cli::Flags),
}
