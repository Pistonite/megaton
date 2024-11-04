use buildcommon::env::Env;
use buildcommon::{errorln, hintln};
use clap::Args;
use derive_more::derive::Deref;
use error_stack::{Result, ResultExt};

use crate::cli::{CommonOptions, TopLevelOptions};
use crate::error::Error;

/// CLI Options for the install command
#[derive(Debug, Clone, PartialEq, Args, Deref)]
pub struct Options {
    /// Pull latest version of the megaton repo with git
    #[clap(short, long)]
    pub update: bool,

    /// Common options
    #[deref]
    #[clap(flatten)]
    pub options: CommonOptions,
}

pub fn run(top: &TopLevelOptions, options: &Options) -> Result<(), Error> {
    let result = run_internal(top, options);
    if result.is_err() {
        if options.update { 
            errorln!("Failed", "Update unsuccessful");
            hintln!("Consider", "Perform a clean installation");
        } else {
            errorln!("Failed", "Install unsuccessful");
        }
    }

    result
}

fn run_internal(top: &TopLevelOptions, options: &Options) -> Result<(), Error> {
    let env = Env::load(top.home.as_deref()).change_context(Error::Install)?;
    if options.update {
    }

    todo!()
}
