// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use cu::pre::*;
use megaton_config::toolchain::HomeEnv;

/// The `toolchain` subcommand
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    #[clap(subcommand)]
    command: SubCmd,
}

#[derive(Debug, clap::Subcommand)]
enum SubCmd {
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
    /// Check toolchain installation
    #[clap(hide = true)]
    Check(cu::cli::Flags),

    /// Invoke cxxbridge, installs if needed
    Cxxbridge(CmdCxxBridge)
}

    /// Invoke cxxbridge, installs if needed
#[derive(Debug, clap::Parser, AsRef)]
#[clap(bin_name = "megaton-toolchain cxxbridge")]
struct CmdCxxBridge {
        /// Generate binding for this RS file
        rs_file: Option<String>,
        /// Generate header instead of CPP
        #[clap(long)]
        header: bool,
        #[clap(flatten)]
    #[as_ref]
        common: cu::cli::Flags,
}

impl Cmd {
    pub fn run(self) -> cu::Result<()> {
                cu::cli::print_to(cu::cli::Target::Stderr);
                cu::cli::animate_to(Some(cu::cli::Target::Stderr));
        let home = HomeEnv::resolve()?;
        match self.command {
            SubCmd::Install { keep, clean, .. } => {
                crate::cxxbridge::install(&home)?;
                crate::rust::install(&home, keep, clean)?;
                Ok(())
            }
            SubCmd::Remove(_) => {
    crate::cxxbridge::remove(&home)?;
    cu::info!("uninstalled cxxbridge");
    crate::rust::remove(&home)?;
    cu::info!("uninstalled megaton rust toolchain");
    Ok(())
            }
            SubCmd::Clean(_) => {
                crate::rust::clean(&home)
            }
            SubCmd::Check(_) => {
    crate::cxxbridge::inspect(&home);
    crate::rust::inspect();
                Ok(())
            }
            SubCmd::Cxxbridge (CmdCxxBridge{rs_file, header, ..}  ) => {
                if rs_file.is_none() && !header {
                    cu::lv::disable_print_time();
                    cu::cli::print_help::<CmdCxxBridge>(false);
                    return Ok(());
                }
                crate::cxxbridge::install(&home)?;
                let mut args = Vec::with_capacity(2);
                if let Some(file) = rs_file {
                    args.push(file);
                }
                if header {
                    args.push("--header".to_string())
                }
                if args.is_empty() {
                }
                crate::cxxbridge::invoke(&home, args)?;
                Ok(())
            }
        }
    }
}
impl AsRef<cu::cli::Flags> for Cmd {
    fn as_ref(&self) -> &cu::cli::Flags {
        match &self.command {
            SubCmd::Install { common, .. } => common,
            SubCmd::Remove(args) => args,
            SubCmd::Clean(args) => args,
            SubCmd::Check(args) => args,
            SubCmd::Cxxbridge( command) => command.as_ref(),
        }
    }
}
