// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::path::Path;

use cu::pre::*;
use derive_more::AsRef;
use flate2::bufread::GzDecoder;

use crate::env::environment;
use config::Flags;

mod check;
mod compile;
mod config;
mod link;
mod rust;

/// Compile and link the megaton project
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
    let env = environment();

    ////////// Load config //////////
    let config = config::load_config(&args.config).context("Failed to load config")?;
    let profile = config.profile.resolve(&args.profile)?;
    let build_config = config.build.get_profile(profile);
    let mut build_flags = Flags::from_config(&build_config.flags);
    let lib_enabled = config.megaton.lib_enabled();
    let profile_target = config.module.target.join("megaton").join(profile);
    cu::fs::make_dir(&profile_target)?;

    // Set up target paths
    let profile_target = profile_target.normalize()?;
    let target_mod = profile_target.join(&config.module.name);
    let target_mod_src = target_mod.join("src");
    let target_mod_include = target_mod.join("include");
    let target_mod_o = target_mod.join("o");
    let compile_db_path = target_mod.join("compiledb.cache");
    let target_lib = profile_target.join("lib");
    cu::fs::make_dir(&target_mod_src)?;
    cu::fs::make_dir(&target_mod_include)?;
    cu::fs::make_dir(&target_mod_o)?;

    if !args.configure {
        make_npdm_json(&target_mod, &config.module.title_id_hex()).await?;
    }

    cu::debug!("Cmd_build: using profile {profile}");

    let mut static_libs = vec![];
    let mut need_link = false;

    if lib_enabled {
        cu::debug!("Cmd_build: libmegaton enabled");
        install_lib_if_needed(&target_lib)?;
    }

    ////////// Build rust //////////
    let rust_ctx = rust::RustCtx::from_config(config.cargo);
    let rust_enabled = rust_ctx.is_some();
    if lib_enabled && let Some(rust_ctx) = rust_ctx {
        let rust_ctx =
            rust_ctx.context("Rust is enabled, but cargo context could not be initialized")?;
        rust_ctx.check_cxx_version()?;

        if !args.configure {
            need_link |= rust_ctx
                .build(&build_flags.cargoflags, &build_flags.rustflags, false)
                .await?;
            static_libs.push(
                rust_ctx
                    .get_output()
                    .context("Failed to get cargo output")?,
            );
        } else if rust_ctx.has_build_script() {
            // run cargo check which calls build script before configuring
            rust_ctx
                .build(&build_flags.cargoflags, &build_flags.rustflags, true)
                .await?;
        }

        need_link |= rust_ctx
            .gen_cxxbridge(&target_mod_src, &target_mod_include)
            .await
            .context("Failed to generate CXX interop files")?;
    }

    ////////// Compile sources //////////
    build_flags.add_includes(env.dkp_includes());

    let mut contexts = vec![];

    // If libmegaton enabled, create library context
    if lib_enabled {
        // Add public library includes
        build_flags.add_includes([target_lib.join("include").into_utf8()?]);

        let mut lib_flags = build_flags.clone();

        // Add nnheaders includes
        lib_flags.add_includes([
            env.dkp_path().join("libnx").join("include").into_utf8()?, // TODO: remove this
            target_lib.join("nnheaders").join("include").into_utf8()?,
        ]);
        cu::hint!("TODO: remove libnx includes");

        lib_flags.add_defines([
            "MEGATON_LIB",
            &format!("MEGART_NX_MODULE_NAME=\"{}\"", &config.module.name),
            &format!("MEGART_NX_MODULE_NAME_LEN={}", &config.module.name.len()),
            &format!("MEGART_TITLE_ID={}", &config.module.title_id),
            &format!("MEGART_TITLE_ID_HEX=\"0x{:016x}\"", &config.module.title_id),
        ]);
        if rust_enabled {
            lib_flags.add_defines(["MEGART_RUST"]);
        }
        let lib_ctx = compile::CompileCtx::new(
            vec![target_lib.join("src")],
            target_mod_o.clone(),
            lib_flags,
        );
        contexts.push(lib_ctx);
    }

    // Create module context
    let mut build_includes = vec![
        target_mod_include.into_utf8()?, // cxxbridge includes
    ];
    for include in build_config.includes {
        build_includes.push(include.normalize_exists()?.into_utf8()?);
    }

    let mut build_sources = vec![
        target_mod_src, // cxxbridge src
    ];
    for source in build_config.sources {
        build_sources.push(source.normalize_exists()?);
    }

    let mut module_flags = build_flags.clone();
    module_flags.add_includes(build_includes);

    let mod_ctx = compile::CompileCtx::new(build_sources, target_mod_o.clone(), module_flags);
    contexts.push(mod_ctx);

    // Compile both contexts
    let compile_commands_path = config.module.compdb.normalize()?;
    let (compiled, mut objects) = compile::compile_all(
        &contexts,
        &compile_db_path,
        &compile_commands_path,
        args.configure,
    )
    .await?;
    need_link |= compiled;

    ////////// Link & Check //////////
    if args.configure {
        cu::info!("Configured build");
        return Ok(());
    }

    let mut libpaths = vec![];
    for libpath in build_config.libpaths {
        libpaths.push(libpath.normalize_exists()?.into_utf8()?);
    }
    build_flags.add_libpaths(libpaths);

    let mut ldscripts = vec![];
    if lib_enabled {
        ldscripts.push(profile_target.join("lib").join("link.ld").into_utf8()?);
    }
    for ldscript in build_config.ldscripts {
        ldscripts.push(ldscript.normalize_exists()?.into_utf8()?);
    }

    let verfile_path = target_mod.join("verfile");
    let entry = config.megaton.entry_point();
    make_verfile(&verfile_path, entry)?;
    build_flags.set_init(entry);
    build_flags.set_version_script(verfile_path.into_utf8()?);
    build_flags.add_ldscripts(ldscripts);
    build_flags.add_libraries(build_config.libraries);

    for obj in build_config.objects {
        objects.push(obj.normalize_exists()?);
    }

    let elf_path = target_mod.join(format!("{}.elf", config.module.name));
    let linked = link::build_elf(
        need_link,
        objects,
        static_libs,
        build_flags.ldflags,
        &elf_path,
        &target_mod.join("linkcmd.cache"),
    )
    .await?;

    let nso_path = target_mod.join(format!("{}.nso", config.module.name));
    if linked || !nso_path.exists() {
        // TODO: check while building nso, delete nso afterwards if check fails
        if let Some(check_config) = config.check {
            let check_config = check_config.get_profile(profile);
            let mut symbol_files = vec![];
            for symbol_file in check_config.symbols {
                symbol_files.push(symbol_file.normalize_exists()?);
            }
            check::check_all(
                &elf_path,
                &check_config.ignore,
                &check_config.disallowed_instructions,
                &symbol_files,
            )
            .await
            .context("Check failed")?;
        }
        link::build_nso(&elf_path, &nso_path).await?;
    } else {
        cu::info!("Up to date")
    }

    Ok(())
}

static LIBRARY_TARGZ: &[u8] = include_bytes!("../../../libmegaton.tar.gz");
static LIBRARY_HASH: &[u8] = include_bytes!("../../../libmegaton_sha256sum");

fn install_lib_if_needed(lib_path: &Path) -> cu::Result<()> {
    if lib_needs_unpacked(lib_path) {
        cu::hint!("Installing libmegaton");
        unpack_lib(lib_path).context("Failed to unpack library archive")?;
    }
    Ok(())
}

/// True if version hash doesn't exist or doesn't match the stored tarball
fn lib_needs_unpacked(lib_path: &Path) -> bool {
    let lib_hash_file = lib_path.join("libmegaton_sha256sum");
    if !&lib_hash_file.exists() {
        return true;
    }
    let Ok(existing_lib_hash) = cu::fs::read(lib_hash_file) else {
        return true;
    };

    LIBRARY_HASH != existing_lib_hash
}

/// Deletes contents of dir and unpacks the library from the stored tarball
fn unpack_lib(lib_path: &Path) -> cu::Result<()> {
    let library_tar = GzDecoder::new(LIBRARY_TARGZ);
    let mut library_archive = tar::Archive::new(library_tar);
    if lib_path.exists() {
        cu::fs::remove_contents(lib_path)?;
    }
    library_archive.unpack(lib_path)?;
    let lib_hash_file = lib_path.join("libmegaton_sha256sum");
    cu::fs::write(lib_hash_file, LIBRARY_HASH)?;

    Ok(())
}

async fn make_npdm_json(output_dir: &Path, title_id_hex: &str) -> cu::Result<()> {
    let mut npdm_data: json::Value = json::parse(include_str!("../../../template.npdm.json"))?;
    npdm_data["title_id"] = json!(format!("0x{}", title_id_hex));

    let main_npdm_json = output_dir.join("main.npdm.json");
    let main_npdm = output_dir.join("main.npdm");

    cu::fs::write_json_pretty(&main_npdm_json, &npdm_data)?;

    environment()
        .npdmtool()
        .command()
        .add(cu::args![&main_npdm_json, &main_npdm])
        .all_null()
        .co_wait_nz()
        .await?;

    cu::info!("Created npdm: {}", main_npdm.try_to_rel().display());
    Ok(())
}

fn make_verfile(path: &Path, entry: &str) -> cu::Result<()> {
    let verfile_before = "{\n\tglobal:\n\n";
    let verfile_after = ";\n\tlocal: *;\n};";
    let verfile_data = format!("{}{}{}", verfile_before, entry, verfile_after);
    if write_if_changed(path, verfile_data.as_bytes())? {
        cu::debug!("Cmd_build: updated verfile");
    } else {
        cu::debug!("Cmd_build: verfile up to date");
    }
    Ok(())
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> cu::Result<bool> {
    let changed = match cu::fs::read(path) {
        Ok(existing) => existing != bytes,
        Err(_) => true,
    };
    if changed {
        cu::fs::write(path, bytes)?;
    }
    Ok(changed)
}
