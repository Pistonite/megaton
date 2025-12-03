// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors



use cu::pre::*;

use crate::env;

mod cmd_build;
use cmd_build::*;
mod cmd_toolchain;
use cmd_toolchain::*;

static LOGO: &str = r#"
 __    __ ______ ______ ______ ______ ______ __   __  
/\ "-./  \\  ___\\  ___\\  __ \\__  _\\  __ \\ "-.\ \ 
\ \ \-./\ \\  __\ \ \__ \\  __ \_/\ \/ \ \/\ \\ \-.  \
 \ \_\ \ \_\\_____\\_____\\_\ \_\\ \_\\ \_____\\_\\"\_\
  \/_/  \/_//_____//_____//_/\/_/ \/_/ \/_____//_/ \/_/"#;

/// Megaton Build Tool
#[derive(clap::Parser, Clone)]
#[clap(before_help(LOGO))]
pub struct CmdMegaton {
    #[clap(subcommand)]
    sub: CmdMegatonSub,

    /// Print the version
    #[clap(short, long)]
    version: bool,
}

#[cfg(test)]
impl CmdMegaton {
    pub fn new_build(config: String) -> Self {
        let sub = CmdBuild::new(config);
        Self { sub: CmdMegatonSub::Build(sub), version: false }
    }
}

impl AsRef<cu::cli::Flags> for CmdMegaton {
    fn as_ref(&self) -> &cu::cli::Flags {
        self.sub.as_ref()
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum CmdMegatonSub {
    /// Print the version and build information
    Version(cu::cli::Flags),
    Build(CmdBuild),
    Toolchain {
        #[clap(subcommand)]
        cmd: CmdToolchain,
    },
}

impl AsRef<cu::cli::Flags> for CmdMegatonSub {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            Self::Version(x) => x,
            Self::Build(cmd) => cmd.as_ref(),
            Self::Toolchain { cmd } => cmd.as_ref(),
        }
    }
}

/// Main entry point to running the `megaton` command
pub fn main(cmd: CmdMegaton) -> cu::Result<()> {
    if cmd.version || matches!(cmd.sub, CmdMegatonSub::Version(_)) {
        print_version();
        return Ok(());
    }
    // safety: program is single threaded at this point
    unsafe { env::init_env()? };
    match cmd.sub {
        CmdMegatonSub::Version(_) => unreachable!(),
        CmdMegatonSub::Build(cmd) => cmd.run(),
        CmdMegatonSub::Toolchain { cmd } => cmd.run(),
    }
}

fn print_version() {
    println!(
        "megaton v{} ({})",
        env!("CARGO_PKG_VERSION"),
        &env::commit()[0..8]
    );
    cu::disable_print_time();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env::set_current_dir, str::FromStr};
    use std::path::PathBuf;

    #[test]
    fn test_build() {
        let mut mod_base_path = PathBuf::new();
        mod_base_path.push(env!("CARGO_MANIFEST_DIR")); // megaton/packages/cli
        mod_base_path = PathBuf::from(mod_base_path.parent().expect("Failed to cd to mod_base path!")); // megaton/packages
        mod_base_path.push("mod_base"); // megaton/packages/mod_base/example_mod
        mod_base_path.push("example-mod");
        let res = set_current_dir(mod_base_path.clone());
        assert!(res.is_ok(), "Failed to cd to mod_base path {:?}! Error: {:?}", mod_base_path, res.unwrap_err());
        println!("Changed directory to: {:?}", std::env::current_dir());
        let cmd = CmdMegaton::new_build("Megaton.toml".to_string());
        let res = main(cmd);
        assert!(res.is_ok(), "{:?}", res.unwrap_err());
    }
}
