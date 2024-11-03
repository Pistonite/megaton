use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use error_stack::Result;

use buildcommon::env::Env;
use buildcommon::{print, system};



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
    Init(CommonOptions),
    /// Build a project
    Build(CommonOptions),
    /// Clean project outputs
    Clean(CommonOptions),
    /// Check the environment and installation status of 
    /// megaton, dependent tools and toolchain/libraries
    ///
    /// The paths found will be cached for faster lookup in the future
    Checkenv(CommonOptions),
    /// Pull the latest version of the megaton repo and update the build tool
    Update(CommonOptions),
    /// Rustc options
    Rustc(CommonOptions),
}

impl std::ops::Deref for Command {
    type Target = CommonOptions;

    fn deref(&self) -> &Self::Target {
        match self {
            Command::Init(x) => x,
            Command::Build(x) => x,
            Command::Clean(x) => x,
            Command::Checkenv(x) => x,
            Command::Update(x) => x,
            Command::Rustc(x) => x,
        }
    }
}

/// Common options for all commands
#[derive(Debug, Clone, PartialEq, Args)]
struct CommonOptions {
    /// Enable verbose output
    #[clap(short = 'v', long)]
    pub verbose: bool,
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
    print::auto_color();
    let arg = Cli::parse();
    if arg.command.verbose {
        print::verbose_on();
    }
    match arg.command {
        Command::Checkenv(_) => {
            let env = Env::check(arg.home)?;
            env.save()?;
        }
        _ => todo!()
    }

    Ok(())
}
