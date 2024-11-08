use std::io::BufRead;
use std::path::Path;
use std::time::Instant;

use buildcommon::prelude::*;
use buildcommon::system::Command;
use buildcommon::env::{self, ProjectEnv};

use clap::Args;
use derive_more::derive::Deref;

use crate::cli::{CommonOptions, TopLevelOptions};
use crate::error::Error;

mod builder;
mod checker;
mod config;
mod build_project;

use config::Config;

/// CLI Options for the build command
#[derive(Debug, Clone, PartialEq, Args, Deref)]
pub struct Options {
    /// Select profile to build
    #[clap(short, long, default_value = "none")]
    pub profile: String,

    /// Only build the compile database (compile_commands.json)
    #[clap(short = 'c', long)]
    pub compdb: bool,

    /// Build libmegaton before building the current project
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

pub fn run(top: &TopLevelOptions, options: &Options) -> Result<(), Error> {
    let start_time = Instant::now();

    let root = env::find_root(&top.dir).change_context(Error::Config)?;
    let megaton_toml = root.join("Megaton.toml");
    let config = Config::from_path(&megaton_toml)?;
    let profile = config.profile.resolve(options.profile.as_str())?;
    let env =
        ProjectEnv::load(
        top.home.as_deref(), root, profile, &config.module.name).change_context(Error::Config)?;

    if options.lib {
        // we can technically prep the project while buliding
        // libmegaton, however, that's not a common workflow worth the 
        // optimization effor
        build_lib(&env)?;
    }

    let artifact = build_project::run(
        &env,
        &config,
        profile,
        &megaton_toml,
        options)?;

    let elapsed = start_time.elapsed();
    infoln!(
        "Finished",
        "{} in {:.2}s",
        artifact,
        elapsed.as_secs_f32()
    );

    Ok(())
}

error_context!(BuildLib, |r| -> Error {
    errorln!("Failed", "Building libmegaton");
    r.change_context(Error::BuildLib)
});

fn build_lib(env: &ProjectEnv) -> ResultIn<(), BuildLib> {
    infoln!("Configuring", "libmegaton");

    let ninja_build_dir = env.megaton_home.join("lib").join("build");
    let build_ninja = ninja_build_dir.join("build.ninja");

    // configure ninja
    Command::new(&env.cargo)
        .current_dir(&env.megaton_home)
        .args(args!["run", "--quiet", "--bin", "megaton-lib-configure", "--", &build_ninja])
        .spawn()?
        .wait()?
        .check()?;

    let mut child = Command::new(&env.ninja)
        .args(args!["-C", ninja_build_dir])
        .piped()
        .spawn()?;

    let handle = child.take_stdout().map(|stdout| {
        std::thread::spawn(move || {
            for line in stdout.lines().map_while(|r| r.ok()) {
                // pretty print ninja progress
                let i = match line.find(']') {
                    Some(i) => i,
                    None => continue,
                };

                let line = line[i+1..].trim_start();

                let s = match line.find(' ') {
                    Some(s) => s,
                    None => continue,
                };

                let status = &line[..s];
                let path = Path::new(&line[s+1..]).rebase(&ninja_build_dir);
                infoln!(status, "{}", path.display());
            }
        })
    });

    let mut child = child.wait()?;
    let _ = handle.map(|h| h.join());
    let result = child.check();
    if result.is_err() {
        child.dump_stderr("Error");
    }
    result?;

    infoln!("Finished", "libmegaton");

    Ok(())
}
