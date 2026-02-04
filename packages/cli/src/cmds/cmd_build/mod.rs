// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::path::Path;

use cu::pre::*;
use derive_more::AsRef;

use config::Flags;
use rust_crate::RustCrate;

mod compile;
mod compile_db;
mod config;
mod rust_crate;

// // The compressed library source archive. Extracted and compiled by the build command
// static LIBRARY_TARGZ: &[u8] = include_bytes!("../../../libmegaton.tar.gz");
//
// fn unpack_lib(lib_root_path: &Path) -> cu::Result<()> {
//     let library_tar = GzDecoder::new(LIBRARY_TARGZ);
//     let mut library_archive = tar::Archive::new(library_tar);
//     library_archive.unpack(lib_root_path)?;
//     Ok(())
// }

/// `megaton` project
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
    pub async fn run(self) -> cu::Result<()> {
        run_build(self).await
    }
}

async fn run_build(args: CmdBuild) -> cu::Result<()> {
    cu::hint!("run with -v to see additional output");

    ////////// Load config //////////
    let config = config::load_config(&args.config).context("failed to load config")?;
    cu::debug!("{config:#?}");

    let profile = config.profile.resolve(&args.profile)?;
    cu::debug!("profile: {profile}");

    let build_config = config.build.get_profile(profile);
    let build_flags = Flags::from_config(&build_config.flags);
    cu::debug!("build flags: {build_flags:#?}");

    let entry = config.megaton.entry_point();
    cu::debug!("entry={entry}");

    let title_id_hex = config.module.title_id_hex();
    cu::debug!("title_id_hex={title_id_hex}");

    ////////// Build rust //////////
    let rust_crate = RustCrate::from_config(config.cargo)?;
    if !rust_crate.is_none() {
        let rust_crate = rust_crate.unwrap();
        cu::debug!("cargo manifest: {}", rust_crate.manifest.display());

        rust_crate
            .build(&build_flags.cargoflags, &build_flags.rustflags)
            .await?;

        cu::debug!("cargo output={}", rust_crate.get_output_path()?.display());

        rust_crate.gen_cxxbridge().await?;
    }

    Ok(())
}
