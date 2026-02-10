// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use cu::pre::*;
use derive_more::AsRef;

use config::Flags;
use flate2::bufread::GzDecoder;
use generate::generate_cxx_bridge_src;

mod check;
mod compile;
mod config;
mod generate;
mod scan;

use scan::discover_source;

use crate::cmds::cmd_build::{
    compile::{CompileDB, build_nso},
    config::CargoConfig,
};

static LIBRARY_TARGZ: &[u8] = include_bytes!("../../../libmegaton.tar.gz");

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

// <target>/megaton
//   - <profile>/: per-profile build files
//     - lib/: where megaton emits it's own library file and build files
//
//     - <module>/: per-module build files
//       - include/: generated header files
//         - rust/cxx.h:
//       - src/cxxbridge:
//       - o/: output object files
//       - <module>.elf
//       - <module>.nso
//       - ...: other output files and caches

#[allow(dead_code)]
struct BTArtifacts {
    project_root: PathBuf,
    target: PathBuf, // ./target/megaton
    profile: String,
    module: String,

    module_root: PathBuf,
    module_obj: PathBuf, // module/o
    module_src: PathBuf,
    module_include: PathBuf,
    // module_cxxbridge_src: PathBuf,
    // module_cxxbridge_include: PathBuf,
    elf_path: PathBuf,
    nso_path: PathBuf,

    lib_root: PathBuf,
    cxxbridge_bin: PathBuf,
    lib_src: PathBuf,
    lib_include: PathBuf,
    lib_linkldscript: PathBuf,
    verfile_path: PathBuf,

    compdb_path: PathBuf,
    command_log_path: PathBuf, // lib_staticlib: PathBuf, // lib/libmegaton.a
}

impl BTArtifacts {
    pub fn new(target_path: PathBuf, module_name: &str, profile_name: &str) -> Self {
        let megaton_root = target_path.join("megaton");
        let profile_root = megaton_root.join(profile_name);
        let module_root = profile_root.join(module_name);
        let lib_root = profile_root.join("lib");
        let lib_src = lib_root.join("src");

        Self {
            project_root: PathBuf::from(".").canonicalize().unwrap(),
            target: target_path.clone(),
            module: module_name.to_owned(),
            profile: profile_name.to_owned(),
            module_root: module_root.clone(),
            module_obj: module_root.join("o"),
            module_src: module_root.join("src"),
            module_include: module_root.join("include"),
            // module_cxxbridge_include: module_root.join("include").join("cxxbridge"),
            // module_cxxbridge_src: module_root.join("src").join("cxxbridge"),
            elf_path: module_root.join(format!("{}.elf", module_name)),
            nso_path: module_root.join(format!("{}.nso", module_name)),

            lib_root: lib_root.clone(),
            cxxbridge_bin: lib_root.join("bin/cxxbridge"),
            lib_src: lib_src.clone(),
            lib_include: lib_root.join("include"),
            lib_linkldscript: lib_root.join("link.ld"),
            verfile_path: lib_root.join("verfile"),

            compdb_path: profile_root.join("compdb.cache"),
            command_log_path: profile_root.join("command_log.txt"),
        }
    }
}

// #[allow(dead_code)]
// struct RustCrate {
//     manifest: PathBuf,
//     got_built: bool,
// }
// A rust crate that will be built as a component of the megaton lib or the mod
pub struct RustCrate {
    pub manifest: PathBuf,
    pub target_path: PathBuf, // Not necessarily the same as the Megaton target dir
    pub source_paths: Vec<PathBuf>,
    pub header_suffix: String,
}

impl RustCrate {
    /// Gets the crate based on the cargo config. Returns `Ok(None)`. Errors if
    /// cargo is explicitly enabled, but couldn't be be found for some reason.
    pub fn from_config(cargo: CargoConfig) -> cu::Result<Option<Self>> {
        let manifest = cargo
            .manifest
            .unwrap_or(CargoConfig::default_manifest_path());

        match cargo.enabled {
            None => Ok(RustCrate::new(&manifest, cargo.sources, cargo.header_suffix).ok()),
            Some(true) => Ok(Some(
                RustCrate::new(&manifest, cargo.sources, cargo.header_suffix)
                    .context("Cargo enabled, but failed to find the crate")?,
            )),
            Some(false) => Ok(None),
        }
    }

    fn new(manifest_path: &Path, sources: Vec<PathBuf>, header_suffix: String) -> cu::Result<Self> {
        let manifest = manifest_path.to_owned().canonicalize().context(format!(
            "Could not find Cargo.toml at {:?}",
            manifest_path.display()
        ))?;

        let crate_root = manifest.parent().unwrap();

        let source_paths = sources
            .iter()
            .map(|rel_path| crate_root.join(rel_path))
            .collect::<Vec<_>>();

        // This should always be target, even if the megaton target dir is differnt
        let target_path = crate_root.join("target");

        Ok(Self {
            manifest,
            target_path,
            source_paths,
            header_suffix,
        })
    }

    pub fn build(&mut self, build_flags: &Flags) -> cu::Result<()> {
        cu::info!("Building rust crate!");
        let cargo = cu::which("cargo").context("cargo executable not found")?;

        let mut command = cargo
            .command()
            .add(cu::args![
                "+megaton",
                "build",
                "--manifest-path",
                &self.manifest,
            ])
            .stdin_null()
            .stdoe(cu::pio::inherit());

        command = command.args(&build_flags.cargoflags);
        command = command.env("RUSTFLAGS", build_flags.rustflags.clone());

        let exit_code = command.spawn()?.wait()?;
        if !exit_code.success() {
            return Err(cu::Error::msg(format!(
                "Cargo build failed with exit status {:?}",
                exit_code
            )));
        }

        Ok(())
    }

    pub fn get_source_folder(&self) -> Vec<PathBuf> {
        vec![PathBuf::from("src")]
    }

    pub fn get_source_files(&self) -> cu::Result<Vec<PathBuf>> {
        let source_dirs = self.get_source_folder();
        let mut source_files: Vec<PathBuf> = vec![];
        for dir in source_dirs {
            let mut walk = cu::fs::walk(dir)?;
            while let Some(entry) = walk.next() {
                let p = entry?.path();
                if p.extension().is_some_and(|e| e == "rs") {
                    source_files.push(p);
                }
            }
        }
        Ok(source_files)
    }

    pub fn get_output_path(&self, target_path: &Path) -> cu::Result<PathBuf> {
        // Assuming cargo is in release mode
        let rel_path = target_path.join("aarch64-unknown-hermit").join("release");
        let name = &cu::fs::read_string(&self.manifest)
            .unwrap()
            .parse::<toml::Table>()
            .unwrap();

        let name = &name["package"]["name"].as_str().unwrap();
        let name = name.replace("-", "_");
        let filename = format!("lib{name}.a");
        let path = rel_path.join(filename).canonicalize()?;

        Ok(path)
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

fn unpack_lib(lib_root_path: &Path) -> cu::Result<()> {
    let library_tar = GzDecoder::new(LIBRARY_TARGZ);
    let mut library_archive = tar::Archive::new(library_tar);
    library_archive.unpack(lib_root_path)?;
    Ok(())
}

impl Flags {
    fn add_c_cpp_flags(&self, new_flags: Vec<String>) -> Self {
        let mut flags = self.clone();
        flags.cflags.extend(new_flags.clone());
        flags.cxxflags.extend(new_flags);
        flags
    }
}

fn build_lib(
    config: &config::Config,
    build_flags: &Flags,
    btart: &BTArtifacts,
    compdb: &mut CompileDB,
) -> cu::Result<bool> {
    // build lib
    let mut compiler_did_something = false;
    // build lib
    let module_name = format!("-D MEGART_NX_MODULE_NAME={:?}", config.module.name);
    let module_name_len = format!("-D MEGART_NX_MODULE_NAME_LEN={:?}", module_name.len());
    let title_id = format!("-D MEGART_TITLE_ID={:?}", config.module.title_id);
    let title_id_hex = format!("-D MEGART_TITLE_ID_HEX={:016x}", config.module.title_id);
    let mut lib_build_flags = vec![
        module_name,
        module_name_len,
        title_id,
        title_id_hex,
        "-DMEGATON_LIB".to_string(),
    ];
    if config.cargo.enabled.is_some_and(|e| e) {
        lib_build_flags.push("-DMEGART_RUST".to_string());
        lib_build_flags.push("-DMEGART_RUST_MAIN".to_string());
    }
    let lib_build_flags = build_flags.add_c_cpp_flags(lib_build_flags);
    discover_source(btart.lib_src.as_path())
        .unwrap_or_default()
        .iter()
        .for_each(|src| {
            let compilation_occurred = src
                .compile(
                    &lib_build_flags,
                    vec![
                        btart.lib_include.display().to_string(),
                        String::from("/opt/devkitpro/libnx/include"),
                    ],
                    compdb,
                    &btart.module_obj,
                )
                .inspect_err(|e| cu::error!("Failed to compile! {:?}", e))
                .unwrap();
            compiler_did_something = compiler_did_something || compilation_occurred;
        });
    Ok(compiler_did_something)
}

fn run_build(args: CmdBuild) -> cu::Result<()> {
    // Load config stuff
    let config = config::load_config(&args.config).context("failed to load config")?;
    cu::hint!("run with -v to see additional output");
    cu::debug!("{config:#?}");

    let profile = config.profile.resolve(&args.profile)?;
    cu::debug!("profile: {profile}");

    let build_config = config.build.get_profile(profile);

    let entry = config.megaton.entry_point();
    cu::debug!("entry={entry}");

    let title_id_hex = config.module.title_id_hex();
    cu::debug!("title_id_hex={title_id_hex}");

    let build_flags = Flags::from_config(&build_config.flags);
    cu::debug!("build flags: {build_flags:#?}");

    let target = &config.module.target;
    let megaton_root = target.join("megaton");
    cu::fs::make_dir(&megaton_root)?;
    let bt_artifacts =
        BTArtifacts::new(target.canonicalize().unwrap(), &config.module.name, profile);

    // Build Library
    cu::fs::make_dir(&bt_artifacts.lib_root)?;
    unpack_lib(&bt_artifacts.lib_root).context("Failed to unpack library")?;

    cu::fs::make_dir(&bt_artifacts.module_root)?;

    cu::info!("Reading CompileDB");
    let mut compdb: CompileDB = if !bt_artifacts.compdb_path.exists() {
        File::create(&bt_artifacts.compdb_path)?;
        CompileDB::default()
    } else {
        json::read(
            cu::fs::read(&bt_artifacts.compdb_path)
                .context("Failed to read compdb.cache")?
                .as_slice(),
        )
        .unwrap_or_default()
    };

    let mut compiler_did_something = if config.megaton.library {
        build_lib(&config, &build_flags, &bt_artifacts, &mut compdb)
            .context("Failed to build library")?
    } else {
        false
    };

    let mut sources = build_config.sources.clone();
    sources.push(bt_artifacts.module_src.display().to_string());
    let mut includes = build_config.includes.clone();
    includes.push(bt_artifacts.module_include.display().to_string());
    includes.push(bt_artifacts.lib_include.display().to_string());
    includes.push("/opt/devkitpro/libnx/include".to_owned());

    let mut rust_staticlib_path: Option<PathBuf> = None;
    let rust_crate = RustCrate::from_config(config.cargo.clone())?;
    if let Some(mut rust_crate) = rust_crate {
        rust_crate.build(&build_flags).unwrap();
        //compiler_did_something = compiler_did_something;

        cu::info!("Generating cxx bridge src!");
        generate_cxx_bridge_src(&rust_crate, &bt_artifacts)?;

        // sources.push(bt_artifacts.module_cxxbridge_src.display().to_string());
        // includes.push(bt_artifacts.module_cxxbridge_include.display().to_string());
        rust_staticlib_path = rust_crate
            .get_output_path(&bt_artifacts.target)
            .inspect_err(|e| {
                panic!("Failed to get output rust output path!: {}", e);
            })
            .ok();
    }

    cu::info!("Sources: {:#?}", sources);

    sources
        .iter()
        .flat_map(|src| {
            // todo: inspect and handle errs
            discover_source(PathBuf::from(src).as_path()).unwrap_or_default()
        })
        .for_each(|src| {
            let compilation_occurred = src
                .compile(
                    &build_flags,
                    includes.clone(),
                    &mut compdb,
                    &bt_artifacts.module_obj,
                )
                .inspect_err(|e| cu::error!("Failed to compile! {:?}", e))
                .unwrap();
            compiler_did_something = compiler_did_something || compilation_occurred;
        });

    cu::info!("linking!");
    let link_result = compile::relink(
        &bt_artifacts,
        &mut compdb,
        &config,
        &build_flags,
        &build_config,
        rust_staticlib_path,
        compiler_did_something,
    );
    let link_succeeded = link_result.is_ok();
    if let Err(e) = link_result {
        cu::info!("Error during linking: {:?}", e);
    } else if let Ok(did_relink) = link_result
        && !did_relink
    {
        cu::info!("Skipping relinking.");
    }

    if link_succeeded {
        let elf_path = bt_artifacts
            .module_root
            .join(format!("{}.elf", &config.module.name));
        let nso_path = bt_artifacts
            .module_root
            .join(format!("{}.nso", &config.module.name));
        let _ = build_nso(&elf_path, &nso_path)
            .inspect_err(|e| cu::error!("Failed to build NSO: {}", e));

        if let Some(check) = &config.check {
            let check = check.get_profile(profile);
            let symbols = check::load_known_symbols(&bt_artifacts, &check).unwrap_or_default();
            let res = check::check_symbols(&elf_path, symbols, &check).unwrap();
            if !res.is_empty() {
                cu::error!("Check: Missing symbols found: {:#?}", res);
            } else {
                cu::info!("Check: No missing symbols found");
            }

            let res = check::check_instructions(&elf_path, &check).unwrap();
            if !res.is_empty() {
                cu::error!("Check: Disallowed instructions found: {:#?}", res);
            } else {
                cu::info!("Check: No disallowed instructions found");
            }
        }
    }

    let _ = compdb
        .save(&bt_artifacts.compdb_path)
        .inspect_err(|e| cu::error!("Failed to save compdb.cache! {}", e));
    let _ = compdb
        .save_command_log(&bt_artifacts.command_log_path)
        .inspect_err(|e| cu::error!("Failed to save command log! {}", e));

    compdb.save_cc_json(PathBuf::from(config.module.compdb).as_ref())?;

    Ok(())
}
