
use cu::pre::*;

use crate::env;
use crate::config::{self, BASE_PROFILE, Flags};
use crate::buildsys::{self, BuildArgs, rust, compile, link, check, miscfile};

pub async fn run(args: BuildArgs) -> cu::Result<()> {
    let env = env::get();

    ////////// Load config //////////
    let (root_path, manifest_path) = config::get_root_and_manifest(args.config.as_deref())?;

    // FIXME: we don't want this - we want to resolve the right path with root and pass
    // it to different parts of the build system. Otherwise it could bite us down the road
    std::env::set_current_dir(&root_path)?;

    let config = config::load(&manifest_path)?;
    let profile = config.profile.resolve(&args.profile)?;
    if profile != BASE_PROFILE {
        cu::info!("building profile '{profile}'");
    }
    let build_config = config.build.get_profile(profile);
    let mut build_flags = Flags::from_config(&build_config.flags);
    let target_path = {
        let mut p = config.module.target_path(&root_path);
        p.push("megaton");
        p
    };
    let lib_enabled = config.megaton.lib_enabled();
    let lib_unpack_path = target_path.join("lib");

    let lib_unpack_task =  {
        let lib_unpack_path = lib_unpack_path.clone();
        cu::co::spawn(async move {
            if lib_enabled {
                buildsys::unpack_megaton_lib(&lib_unpack_path).await?
            }
            cu::Ok(())
        })
    };
    let profile_target_path = target_path.join(profile);
    cu::fs::make_dir(&profile_target_path)?;
    
    // Set up target paths
    // TODO: probably don't need to do them at this time?
    let target_mod = profile_target_path.join(&config.module.name);
    let target_mod_src = target_mod.join("src");
    let target_mod_include = target_mod.join("include");
    let target_mod_o = target_mod.join("o");
    let compile_db_path = target_mod.join("compiledb.cache");
    cu::fs::make_dir(&target_mod_src)?;
    cu::fs::make_dir(&target_mod_include)?;
    cu::fs::make_dir(&target_mod_o)?;
    
    if !args.configure {
        miscfile::make_npdm(&target_mod, &config.module.title_id_hex()).await?;
    }
    
    let mut static_libs = vec![];
    let mut need_link = false;
    
    // TODO: overhaul the dependency graph to take advantage of async
    lib_unpack_task.co_join().await??;



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
                    .await
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
        build_flags.add_includes([lib_unpack_path.join("include").into_utf8()?]);
    
        let mut lib_flags = build_flags.clone();
    
        // Add nnheaders includes
        lib_flags.add_includes([
            env.dkp_path().join("libnx").join("include").into_utf8()?, // TODO: remove this
            lib_unpack_path.join("nnheaders").join("include").into_utf8()?,
        ]);
        // cu::hint!("TODO: remove libnx includes");
    
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
            vec![lib_unpack_path.join("src")],
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
    let compile_commands_path = config.module.compdb_path(&root_path);
    let (compiled, mut objects) = compile::compile_all(
        &contexts,
        &compile_db_path,
        &compile_commands_path,
        args.configure,
        env
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
        ldscripts.push(lib_unpack_path.join("link.ld").into_utf8()?);
    }
    for ldscript in build_config.ldscripts {
        ldscripts.push(ldscript.normalize_exists()?.into_utf8()?);
    }
    
    let verfile_path = target_mod.join("verfile");
    let entry = config.megaton.entry_point();
    miscfile::make_verfile(&verfile_path, entry)?;
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



