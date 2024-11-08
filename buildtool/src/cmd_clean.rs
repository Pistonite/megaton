use buildcommon::env::Env;
use buildcommon::prelude::*;

use buildcommon::env;
use clap::Args;
use derive_more::derive::Deref;
use error_stack::{Result, ResultExt};

use crate::cli::{CommonOptions, TopLevelOptions};
use crate::error::Error;

/// CLI Options for the clean command
#[derive(Debug, Clone, PartialEq, Args, Deref)]
pub struct Options {
    /// Only clean the selected profile
    ///
    /// All profiles are cleaned by default
    #[clap(short, long)]
    pub profile: Option<String>,

    /// Also clean libmegaton build output
    #[clap(short = 'L', long, conflicts_with = "profile")]
    pub lib: bool,

    /// Common options
    #[deref]
    #[clap(flatten)]
    pub options: CommonOptions,
}

pub fn run(top: &TopLevelOptions, clean: &Options) -> Result<(), Error> {
    let root = env::find_root(&top.dir).change_context(Error::Config)?;
    let target = root.join("target").into_joined("megaton");
    let output = match &clean.profile {
        Some(profile) => target.into_joined(profile),
        None => target,
    };

    if clean.lib {
        let env = Env::load(top.home.as_deref()).change_context(Error::Config)?;
        let lib_root = env.megaton_home.join("lib").into_joined("build");

        
        system::remove_directory(lib_root.join("bin")).change_context(Error::Clean)?;
        system::remove_file(lib_root.join("compile_commands.json")).change_context(Error::Clean)?;
        system::remove_file(lib_root.join("build.ninja")).change_context(Error::Clean)?;
        system::remove_file(lib_root.join(".ninja_log")).change_context(Error::Clean)?;
        system::remove_file(lib_root.join(".ninja_deps")).change_context(Error::Clean)?;
        infoln!("Cleaned", "libmegaton");
    }

    match system::remove_directory(&output) {
        Ok(_) => {
            infoln!("Cleaned", "{}", output.rebase(&root).display());
            Ok(())
        }
        Err(e) => {
            errorln!(
                "Failed",
                "Cannot remove '{}'",
                output.rebase(&root).display()
            );
            Err(e).change_context(Error::Clean)
        }
    }
}
