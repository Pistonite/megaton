use clap::Args;
use derive_more::derive::Deref;
use error_stack::Result;

use crate::cli::{CommonOptions, TopLevelOptions};
use crate::error::Error;

mod builder;
mod checker;
mod config;
mod run_impl;

/// CLI Options for the build command
#[derive(Debug, Clone, PartialEq, Args, Deref)]
pub struct Options {
    /// Select profile to build
    #[clap(short, long, default_value = "none")]
    pub profile: String,

    /// Only build the compile database (compile_commands.json)
    #[clap(short = 'D', long)]
    pub compdb: bool,

    /// Build libmegaton instead of the current project
    #[clap(
        short = 'L',
        long,
        conflicts_with = "profile",
        conflicts_with = "compdb"
    )]
    pub lib: bool,

    /// Common options
    #[deref]
    #[clap(flatten)]
    pub options: CommonOptions,
}

pub fn run(top: &TopLevelOptions, build: &Options) -> Result<(), Error> {
    run_impl::run(top.home.as_deref(), &top.dir, build)
}
