// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use cu::pre::*;

/// Manage the custom `megaton` Rust toolchain
#[derive(Debug, Clone, clap::Subcommand)]
pub enum CmdToolchain {
    /// Check the installation status of the toolchain
    Check(cu::cli::Flags),
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

impl AsRef<cu::cli::Flags> for CmdToolchain {
    fn as_ref(&self) -> &cu::cli::Flags {
        match &self {
            Self::Check(args) => args,
            Self::Install { common, .. } => common,
            Self::Remove(args) => args,
            Self::Clean(args) => args,
        }
    }
}

impl CmdToolchain {
    pub fn run(self) -> cu::Result<()> {
        match self {
            Self::Install { keep, clean, .. } => megaton_toolchain_build::cmd::install(keep, clean),
            Self::Check(_) => {
                cu::lv::disable_print_time();
                megaton_toolchain_build::cmd::check()
            }
            Self::Remove(_) => megaton_toolchain_build::cmd::remove(),
            Self::Clean(_) => 
            megaton_toolchain_build::cmd::clean(),
        }
    }
}
