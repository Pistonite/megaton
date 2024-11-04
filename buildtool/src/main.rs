use std::process::ExitCode;

use buildcommon::hintln;
use clap::Parser;
use error_stack::Result;

mod cli;
use cli::{Cli, Command};

mod cmd_build;
mod cmd_checkenv;
mod cmd_clean;
mod cmd_install;

mod error;
use error::Error;

fn main() -> ExitCode {
    let cli = Cli::parse();
    cli.apply_print_options();
    if let Err(e) = main_internal(&cli) {
        if cli.is_trace_on() {
            eprintln!("error: {:?}", e);
        } else if !matches!(cli.command, Command::Build(_)) {
            if cli.is_verbose_on() {
                hintln!("Consider", "Running with --trace for more information");
            } else {
                hintln!("Consider", "Running with --verbose or --trace for more information");
            }
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn main_internal(cli: &Cli) -> Result<(), Error> {
    match &cli.command {
        Command::Checkenv(_) => cmd_checkenv::run(&cli.top)?,
        Command::Build(build) => cmd_build::run(&cli.top, &build)?,
        Command::Clean(clean) => cmd_clean::run(&cli.top, &clean)?,
        _ => todo!()
    }

    Ok(())
}
