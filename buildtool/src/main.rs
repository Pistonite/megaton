use std::process::ExitCode;

use clap::{Parser, Subcommand};
use error_stack::Result;

use buildcommon::{env::Env, system};



/// 
#[derive(Debug, Clone, PartialEq, Parser)]
#[clap(version, bin_name="megaton")]
struct Cli {
    /// Change the directory to run in
    #[clap(short('C'), long, default_value = ".")]
    pub dir: String,

    /// Set MEGATOM_HOME.
    ///
    /// Used by the shim script for passing in the path directly
    /// so the tool doesn't need to query it
    #[clap(short('H'), long, hide(true))]
    pub home: Option<String>,

    /// Subcommand
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Subcommand)]
enum Command {
    /// Create a new project
    Init,
    /// Build a project
    Build,
    /// Clean project outputs
    Clean,
    /// Check the environment and installation status of 
    /// megaton, dependent tools and toolchain/libraries
    ///
    /// The paths found will be cached for faster lookup in the future
    Checkenv,
    /// Pull the latest version of the megaton repo and update the build tool
    Update,
    /// Library options
    Library,
    /// Rustc options
    Rustc,
}

fn main() -> ExitCode {
    if let Err(e) = main_internal() {
        eprintln!("error: {:?}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn main_internal() -> Result<(), system::Error> {
    let arg = Cli::parse();
    match arg.command {
        Command::Checkenv => {
            let env = Env::check(arg.home)?;
            env.save()?;
        }
        _ => todo!()
    }

    Ok(())
}
