use buildcommon::prelude::*;

use std::process::ExitCode;

use clap::Parser;

mod cli;
use cli::{Cli, Command};

mod cmd_build;
mod cmd_checkenv;
mod cmd_clean;
mod cmd_init;
mod cmd_install;

mod error;
use error::Error;

fn main() -> ExitCode {
    let cli = Cli::parse();
    cli.apply_print_options();
    if let Err(e) = main_internal(&cli) {
        if cli.is_trace_on() {
            eprintln!("error: {:?}", e);
        } else if cli.command.show_fatal_error_message() {
            errorln!("Fatal", "Error: {}", e);
            if cli.is_verbose_on() {
                hintln!("Consider", "Running with --trace for more information");
            } else {
                hintln!(
                    "Consider",
                    "Running with --verbose or --trace for more information"
                );
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
        Command::Build(options) => cmd_build::run(&cli.top, options)?,
        Command::Clean(options) => cmd_clean::run(&cli.top, options)?,
        Command::Install(options) => cmd_install::run(&cli.top, options)?,
        Command::Init(_) => cmd_init::run(&cli.top.dir)?,
        _ => todo!(),
    }

    Ok(())
}
