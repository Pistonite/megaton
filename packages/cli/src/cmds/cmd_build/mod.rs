// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::path::PathBuf;

use cu::pre::*;
use derive_more::AsRef;

use config::Flags;
use generate::generate_cxx_bridge_src;

mod compile;
mod config;
mod generate;
mod link;
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

    pub fn build(&mut self, build: config::Build) -> cu::Result<()> {
        let cargo = cu::which("cargo").context("cargo executable not found")?;
        let rust_flags = match build.flags.rust {
            Some(flags) => Some(flags.join(" ")),
            None => None,
        };
        let mut command = cargo
            .command()
            .add(cu::args![
                "+megaton",
                "build",
                "--manifest_path",
                &self.manifest,
            ])
            .stdin_null()
            .stdoe(cu::pio::inherit());
        if let Some(cargo_flags) = build.flags.cargo {
            command = command.args(cargo_flags);
        }
        if let Some(rustc_flags) = rust_flags {
            command = command.env("RUSTFLAGS", rustc_flags);
        }
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

pub fn get_project_root() -> PathBuf {
    PathBuf::from(".").canonicalize().unwrap()
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

    // here are just suppressing the unused warning
    // build_flags.add_defines(["-Dfoo"]);
    // build_flags.add_includes(["-Ifoo"]);
    // build_flags.set_init("foo");
    // build_flags.set_version_script("verfile");
    // build_flags.add_libpaths(["foo"]);
    // build_flags.add_libraries(["foo"]);
    // build_flags.add_ldscripts(["foo"]);

    // Get paths
    let megaton_path = config.module.target.join(PathBuf::from("megaton"));
    let profile_path = megaton_path.join(PathBuf::from(profile));
    let compdb_path = profile_path.join(PathBuf::from("compdb.cache"));
    let module_path = profile_path.join(PathBuf::from(config.module.name));

    let mut compdb = json::read(
        cu::fs::read(compdb_path)
            .context("Failed to read compdb.cache")?
            .as_slice(),
    )?;

    let mut rust_crate = RustCrate::new(PathBuf::from(config.cargo.manifest.unwrap()));
    rust_crate.build(build_config);
    let rust_changed = rust_crate.got_built;

    generate_cxx_bridge_src(rust_crate, &module_path)?;

    // TODO: Find all our other source code
    // for source_dir in build_config.sources:
    // let sources = discover_source(source_dir)

    // TODO: Compile all c/cpp/s
    // for source in sources:
    // compile(sources, source_o_name, build_flags)

    for source_dir in build_config.sources {
        // TODO: Combine these into a single step so sources are compiled (or skipped)
        // as they are discovered.
        let sources = discover_source(PathBuf::from(source_dir).as_ref())
            .context("Failed to scan for sources in {source_dir}")?;
        for source in sources {
            source.compile(&build_flags, &mut compdb)?;
        }
    }

    // TODO: Link all our artifacts and make the nso
    // link(??)

    let crate_changed = true;

    Ok(())
}
