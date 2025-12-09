// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use cu::pre::*;
use derive_more::AsRef;
use std::path::{Path, PathBuf};

mod compile;
mod config;
mod generate;
mod link;
mod scan;

use config::Flags;
// use compile::{compile, compile_rust};
use generate::generate_cxx_bridge_src;

// use scan::{discover_crates, discover_source};


// A source file that can be compiled into a .o file
// struct SourceFile {
//     path: PathBuf,
//     lang: Lang,
// }

// Specifies source language (rust is managed separately)
// enum Lang {
//     C,
//     Cpp,
//     S,
// }

// impl SourceFile {
//     pub fn new(lang: Lang, path: PathBuf) -> Self {
//         Self {
//             path,
//             lang,
//         }
//     }
// }

// A rust crate that will be built as a component of the megaton lib or the mod
#[allow(dead_code)]
struct RustCrate {
    path: PathBuf,
    manifest: RustManifest,
}

#[derive(Serialize, Deserialize, Debug)]
struct RustManifest {
    // TODO: Implement

}

impl RustManifest {
    fn load(_crate_path: &Path) -> Self {
        // TODO: Implement
        Self {}
   }
}

impl RustCrate {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path.clone(),
            manifest: RustManifest::load(&path)
        }
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

    // TODO: Init build environment, load needed stuff from config

    // TODO: Discover the rust crate (if rust enabled)
    // let rust_crate = discover_crate(top_level_source_dir);
    
    // TODO: Build rust crate
    // compile_rust(rust_crate);

    // TODO: Generate cxxbridge headers and sources
    // generate_cxx_bridge_src(rust_crate.src_dir, module_target_path)

    let rust_crate = RustCrate::new(PathBuf::from("packages/modules"));
    let module_target_path: PathBuf = config
    .module
    .target
    .as_ref()
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("target/_test_module"));

    generate_cxx_bridge_src(rust_crate, &module_target_path)?;


    
    // TODO: Find all our other source code
    // for source_dir in build_config.sources:
    // let sources = discover_source(source_dir)

    // TODO: Compile all c/cpp/s
    // for source in sources:
    // compile(sources, source_o_name, build_flags)
    
    // TODO: Link all our artifacts and make the nso
    // link(??)

    Ok(())
}

