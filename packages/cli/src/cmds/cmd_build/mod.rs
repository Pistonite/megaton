// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::{fs::File, path::{Path, PathBuf}};

use cu::pre::*;
use derive_more::AsRef;

use config::Flags;
use generate::generate_cxx_bridge_src;

mod compile;
mod config;
mod generate;
mod scan;

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
use scan::discover_source;

use crate::cmds::cmd_build::compile::CompileDB;

// A rust crate that will be built as a component of the megaton lib or the mod
#[allow(dead_code)]
struct RustCrate {
    manifest: PathBuf,
    got_built: bool,
}

impl RustCrate {
    pub fn new(manifest_path: PathBuf) -> Self {
        Self {
            manifest: manifest_path
                .clone()
                .canonicalize()
                .expect(format!("Could not find Cargo.toml at {:?}", manifest_path).as_str()),
            got_built: true,
        }
    }

    pub fn build(&mut self, build: &config::Build, build_flags: &Flags) -> cu::Result<()> {
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
        self.got_built = true;

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
        let rel_path = target_path
            .join("aarch64-unknown-hermit")
            .join("release");
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

// <target>/megaton
//   - <profile>/: per-profile build files
//     - lib/: where megaton emits it's own library file and build files
//       - include/: megaton's include path
//         - rust/cxx.h: header from cxxbridge
//         - megaton/: megaton headers
//       - src/: megaton's C/C++/S/RS source files
//         - cxxbridge/:
//       - dep/*: megaton's dependent Rust crates
//       - Cargo.toml: generate Cargo.toml workspace shim (this needs a [workspace] section)
//     - <module>/: per-module build files
//       - include/: generated header files
//         - rust/cxx.h:
//       - src/cxxbridge:
//       - o/: output object files
//       - <module>.elf
//       - <module>.nso
//       - ...: other output files and caches

fn unpack_lib() -> cu::Result<()> {
    // TODO
    Ok(())
}

fn build_lib() -> cu::Result<()> {
    //TODO:
    Ok(())
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

    let mut build_flags = Flags::from_config(&build_config.flags);
    cu::debug!("build flags: {build_flags:#?}");

    let megaton_path = config.module.target.join(PathBuf::from("megaton"));
    let profile_path = megaton_path.join(PathBuf::from(profile));
    let compdb_path = profile_path.join(PathBuf::from("compdb.cache"));
    let command_log_path = profile_path.join(PathBuf::from("command_log.txt"));
    let module_path = profile_path.join(PathBuf::from(&config.module.name));
    cu::fs::make_dir(&module_path)?;

    cu::info!("Reading CompileDB");
    let mut compdb: CompileDB = if !compdb_path.exists() {
        cu::info!("{}", compdb_path.display());
        File::create(&compdb_path)?;
        CompileDB::default()
    } else {
        json::read(
            cu::fs::read(&compdb_path)
                .context("Failed to read compdb.cache")?
                .as_slice(),
        )
        .unwrap_or_default()
    };

    let mut rust_crate = RustCrate::new(PathBuf::from(config.cargo.manifest.unwrap()));
    rust_crate.build(&build_config, &build_flags).unwrap();
    let rust_changed = rust_crate.got_built;

    cu::info!("Generating cxx bridge src!");
    generate_cxx_bridge_src(&rust_crate, &module_path)?;

    let mut compiler_did_something = false;
    build_config.sources.iter().map(|src| {
        // todo: inspect and handle errs
        discover_source(PathBuf::from(src).as_path()).unwrap_or(vec![])
    }).flatten().for_each(|src| {
        let compilation_occurred = src.compile(&build_flags, &build_config, &mut compdb,  &module_path)
            .inspect_err(|e| cu::error!("Failed to compile! {:?}", e)).unwrap();
        compiler_did_something = compiler_did_something || compilation_occurred;
    });

    cu::info!("linking!");

    let link_result = compile::relink(&module_path, &mut compdb,  &config.module, &build_flags, &build_config, compiler_did_something);
    if let Err(e) = link_result {
        cu::info!("Error during linking: {:?}", e);
    } else if let Ok(did_relink) = link_result && !did_relink {
        cu::info!("Skipping relinking.");
    }

    let _ = compdb.save(&compdb_path).inspect_err(|e| cu::error!("Failed to save compdb.cache! {}", e));
    let _ = compdb.save_command_log(&command_log_path).inspect_err(|e| cu::error!("Failed to save command log! {}", e));

    Ok(())
}
