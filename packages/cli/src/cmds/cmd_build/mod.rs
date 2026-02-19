// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::path::{Path};

use cu::pre::*;
use derive_more::AsRef;
use flate2::bufread::GzDecoder;

use crate::env::environment;

use compile::{CompileCtx, compile_all};
use config::Flags;
use rust::RustCtx;

mod compile;
mod config;
mod link;
mod rust;

/// `megaton` project
#[derive(Debug, Clone, AsRef, clap::Parser)]
pub struct CmdBuild {
    /// Select profile to build
    #[clap(short, long, default_value = "base")]
    pub profile: String,

    /// Emit configuration files only (such as compile_commands.json),
    /// and do not actually build
    #[clap(short = 'g', long)]
    pub configure: bool,

    /// Specify the location of the config file
    #[clap(short = 'c', long, default_value = "Megaton.toml")]
    pub config: String,

    /// Unpack the library archive before building, even if it already
    /// exists and is up to date
    #[clap(short = 'u', long)]
    pub unpack_lib: bool,

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
    let profile = config.profile.resolve(&args.profile)?;
    let build_config = config.build.get_profile(profile);
    let mut build_flags = Flags::from_config(&build_config.flags);
    let profile_target = config.module.target.join("megaton").join(profile);
    cu::fs::make_dir(&profile_target)?;

    // Set up target paths
    let profile_target = profile_target.normalize()?;
    let target_mod = profile_target.join(&config.module.name);
    let target_mod_src = target_mod.join("src");
    let target_mod_include = target_mod.join("include");
    let target_mod_o = target_mod.join("o");
    let compiledb_path = target_mod.join("compiledb.cache");
    cu::fs::make_dir(&target_mod_src)?;
    cu::fs::make_dir(&target_mod_include)?;
    cu::fs::make_dir(&target_mod_o)?;

    cu::debug!("profile: {profile}");

    let mut need_relink = false;

    ////////// Build rust //////////
    let rust_ctx = RustCtx::from_config(config.cargo);
    let rust_enabled = rust_ctx.is_some();
    if config.megaton.lib_enabled()
        && let Some(rust_ctx) = rust_ctx
    {
        let rust_ctx = rust_ctx?;
        cu::debug!("rust support enabled");
        cu::debug!("cargo manifest: {}", rust_ctx.manifest.display());

        let cargo_changed = rust_ctx
            .build(&build_flags.cargoflags, &build_flags.rustflags)
            .await?;

        if cargo_changed {
            cu::debug!("rust staticlib has changed");
            need_relink = true;
        }

        rust_ctx
            .gen_cxxbridge(&target_mod_src, &target_mod_include)
            .await?;
    }

    ////////// Compile sources //////////
    build_flags.add_includes(environment().dkp_includes());

    let mut contexts = vec![];

    // If libmegaton enabled, create library context
    if config.megaton.lib_enabled() {
        cu::debug!("libmegaton enabled");
        let target_lib = profile_target.join("lib");

        if args.unpack_lib {
            // TODO: Unpack automatically if libary files dont exist
            cu::debug!("unpacking library archive");
            unpack_lib(&target_lib).context("Failed to unpack library archive")?;
        }

        // Add public library includes
        build_flags.add_includes([target_lib.join("include").into_utf8()?]);

        let mut lib_flags = build_flags.clone();

        cu::hint!("TODO: remove libnx includes");
        lib_flags.add_includes([environment()
            .dkp_path()
            .join("libnx")
            .join("include")
            .into_utf8()?]);

        lib_flags.add_defines([
            "MEGATON_LIB",
            &format!("MEGART_NX_MODULE_NAME={}", &config.module.name),
            &format!("MEGART_NX_MODULE_NAME_LEN={}", &config.module.name.len()),
            &format!("MEGART_TITLE_ID={}", &config.module.title_id),
            &format!("MEGART_TITLE_ID_HEX={:016x}", &config.module.title_id),
        ]);
        if rust_enabled {
            lib_flags.add_defines(["MEGART_RUST", "MEGART_RUST_MAIN"]);
        }
        let lib_ctx = CompileCtx::new(
            vec![target_lib.join("src")],
            target_mod_o.clone(),
            lib_flags,
        );
        contexts.push(lib_ctx);
    }

    // Create module context
    let mut build_includes = vec![target_mod_include.into_utf8()?];
    for include in build_config.includes {
        build_includes.push(include.normalize_exists()?.into_utf8()?);
    }
    let mut build_sources = vec![target_mod_src];
    for source in build_config.sources {
        build_sources.push(source.normalize_exists()?);
    }
    let mut module_flags = build_flags.clone();
    module_flags.add_includes(build_includes);

    let mod_ctx = CompileCtx::new(build_sources, target_mod_o.clone(), module_flags);
    contexts.push(mod_ctx);

    // Compile both contexts
    need_relink |= compile_all(&contexts, &compiledb_path).await?;

    ////////// Link & Check //////////
    if need_relink {
        cu::debug!("need relink");
    }

    cu::debug!("title_id={}", config.module.title_id_hex());
    cu::debug!("entry={}", config.megaton.entry_point());

    Ok(())
}

static LIBRARY_TARGZ: &[u8] = include_bytes!("../../../libmegaton.tar.gz");
fn unpack_lib(lib_root_path: &Path) -> cu::Result<()> {
    let library_tar = GzDecoder::new(LIBRARY_TARGZ);
    let mut library_archive = tar::Archive::new(library_tar);
    library_archive.unpack(lib_root_path)?;
    Ok(())
}
