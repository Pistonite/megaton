// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

mod compile;
mod config;
mod generate;
mod link;
mod scan;

use std::{
    fs::{File, Metadata, metadata},
    path::{Path, PathBuf},
};

use cu::pre::*;
use derive_more::AsRef;

use compile::compile;
use config::Config;
use config::Flags;
use scan::discover_source;

// A rust crate that will be built as a component of the megaton lib or the mod
struct RustCrate {
    path: PathBuf,
    manifest: RustManifest,
}

#[derive(Serialize, Deserialize, Debug)]
struct RustManifest {
    // TODO: Implement
}

impl RustManifest {
    fn load(crate_path: &Path) -> Self {
        // TODO: Implement
        Self {}
    }
}

impl RustCrate {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path.clone(),
            manifest: RustManifest::load(&path),
        }
    }
}

// Stores all the info needed for the build process. Things in here would
// otherwise be accessible from the configuration or in the global environment.
// This should be fast so it should just get initialized once and have
// immutable references to it passed where needed.
struct BuildEnvironment {
    target: String,
    module: String,
    profile: String,
    compdb_clangd: String,
    cc_version: String,
    cxx_version: String,
}

impl BuildEnvironment {
    // Initalize a new copy of the environment
    fn init(config: &Config) -> cu::Result<Self> {
        let profile = if config.profile.allow_base {
            String::from("none")
        } else {
            match &config.profile.default {
                Some(name) => name.clone(),
                None => return Err(cu::Error::msg("Failed to get profile name")),
            }
        };
        Ok(Self {
            target: config.module.target.clone().unwrap_or(String::from("target")),
            module: config.module.name.clone(),
            profile,
            compdb_clangd: config.module.compdb.clone().unwrap_or(String::from("compile_commands.json")),

            // TODO: Figure out how to get these at runtime
            cc_version: todo!(),
            cxx_version: todo!(),
        })
    }
}

/// Manage the custom `megaton` Rust toolchain
#[derive(Debug, Clone, AsRef, clap::Parser)]
pub struct CmdBuild {
    /// Select profile to build
    #[clap(short, long, default_value = "none")]
    pub profile: String,

    /// Emit configuration files only (such as compile_commands.json),
    /// and do not actually build
    #[clap(short = 'g', long)]
    pub configure: bool,

    /// Specify the location of the config file
    #[clap(short = 'c', long, default_value = "Megaton.toml")]
    pub config: String,

    #[clap(flatten)]
    #[as_ref]
    common: cu::cli::Flags,
}

impl CmdBuild {
    pub fn run(self) -> cu::Result<()> {
        run_build(self)
    }
}

fn run_build(args: CmdBuild) -> cu::Result<()> {
    let config = config::load_config(&args.config).context("failed to load config")?;
    cu::hint!("run with -v to see additional output");
    cu::debug!("{config:#?}");
    let profile = config.profile.resolve(&args.profile)?;
    // you can mess with different -p flags and config combination
    // to see how the config parsing and validation system work
    cu::debug!("profile: {profile}");
    let build_config = config.build.get_profile(profile);
    let entry = config.megaton.entry_point();
    cu::debug!("entry={entry}");
    let title_id_hex = config.module.title_id_hex();
    cu::debug!("title_id_hex={title_id_hex}");

    let mut build_flags = Flags::from_config(&build_config.flags);
    cu::debug!("build flags: {build_flags:#?}");

    // here are just suppressing the unused warning
    build_flags.add_defines(["-Dfoo"]);
    build_flags.add_includes(["-Ifoo"]);
    build_flags.set_init("foo");
    build_flags.set_version_script("verfile");
    build_flags.add_libpaths(["foo"]);
    build_flags.add_libraries(["foo"]);
    build_flags.add_ldscripts(["foo"]);

    // Init build environment, load needed stuff from config
    let build_env = BuildEnvironment::init(&config)?;

    // TODO: Load cached compdb (or generate a new one)
    let comp_db_cache_path = todo!();

    // Where is serde json?
    let compile_db = json::parse(comp_db_cache_path);

    // TODO: Discover the rust crate (if rust enabled)
    // let rust_crate = discover_crate(top_level_source_dir);

    // TODO: Build rust crate
    // compile_rust(rust_crate);

    // TODO: Generate cxxbridge headers and sources
    // generate_cxx_bridge_src(rust_crate.src_dir, module_target_path)

    for source_dir in build_config.sources {
        // Find all c/cpp/s source code

        // TODO: Combine these into a single step so sources are compiled (or skipped)
        // as they are discovered.
        let sources = discover_source(PathBuf::from(source_dir).as_ref())
            .context("Failed to scan for sources in {source_dir}")?;
        for source in sources {
            compile(&source, &build_flags, &build_env, &mut compile_db)?;
        }
    }

    // TODO: Link all our artifacts and make the nso
    // link(??)

    Ok(())
}
