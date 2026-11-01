use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use cargo_metadata::MetadataCommand;
use cu::pre::*;
use semver::Version;

use megaton_config::{config_file::{Config, RustConfig}, toolchain::{MEGATON_TOOLCHAIN_SELECTOR, RustEnv, ToolchainEnv}};
use megaton_toolchain::cxxbridge;

use crate::path_hash;

#[derive(Debug)]
pub struct RustProject {
    /// If the project uses the cxx crate
    pub has_cxx: bool,
    /// If the project has build scripts
    pub has_build_script: bool,
    /// Output archive path
    pub output_path: PathBuf,
    /// Namespace of the include (same as the crate name)
    pub include_namespace: String,
}

/// Check the cargo manifest for megaton and cxx dependencies.
///
/// The megaton dependency will be added if possible (if not found)
#[cu::context("failed to resolve Rust project context")]
pub fn resolve_project(config: &Config) -> cu::Result<RustProject> {
    // cxx must be declared in the manifest as dependency,
    // so we can skip cargo-metadata if cxx is not found in the manifest
    let manifest_path = &config.rust.cargo_manifest;
    let manifest_dir = manifest_path.parent_abs()?;

    let manifest = cu::check!(cu::fs::read_string(manifest_path), "failed to read project's Cargo.toml")?;
    let manifest = cu::check!(toml::parse::<toml::Table>(&manifest), "failed to parse project's Cargo.toml")?;

    let crate_name = manifest.get("package").and_then(|x| x.as_table()).and_then(|x| x.get("name"))
        .and_then(|x| x.as_str());
    let crate_name = cu::check!(crate_name, "failed to resolve crate name; ensure package.name exists in Cargo.toml")?;

    // resolve output .a file path from cargo
    let output_path = {
        let lib_name = manifest.get("lib").and_then(|x| x.as_table())
        .and_then(|x| x.get("name")).and_then(|x| x.as_str());
        let archive_name = match lib_name {
            None => {
                format!("lib{}.a", crate_name.replace("-", "_"))
            }
            Some(name) => {
                format!("lib{name}.a")
            }
        };
        // cargo flag would include --target-dir to direct to the same target directory
        cu::path!(&(&config.project.target_root) / "aarch64-unknown-hermit" / "release" / archive_name)
    };

    let dependencies = cu::check!(manifest.get("dependencies").and_then(|x| x.as_table()), "cannot find [dependencies] in Cargo.toml")?;

    let has_cxx = dependencies.get("cxx").is_some();
    let has_build_script = cu::path!(&manifest_dir / "build.rs").is_file();

    let expected_megaton_path = cu::path!(&(&config.project.lib_root) / "packages" / "mega-rs");
    let expected_megaton_path = expected_megaton_path.try_to_rel_from(manifest_dir);
    let expected_megaton_path = expected_megaton_path.as_utf8()?;
    enum MegatonDepState {
        NotFound,
        NotSetPath,
        NotRightPath,
        Ok
    }
    let megaton_state = match dependencies.get("megaton") {
        None => MegatonDepState::NotFound,
        Some(megaton) => {
            match megaton.as_table() {
                None => MegatonDepState::NotSetPath,
                Some(megaton) => {
                    match megaton.get("path") {
                        None => MegatonDepState::NotSetPath,
                        Some(p) => {
                            match p.as_str() {
                                None => MegatonDepState::NotRightPath,
                                Some(p) => {
                                    if p == expected_megaton_path {
                                        MegatonDepState::NotRightPath
                                    } else {
                                        MegatonDepState::Ok
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    let megaton_not_ok = match megaton_state {
        MegatonDepState::NotFound => {
            cu::warn!("cannot find 'megaton' in Cargo.toml");
            true
        }
        MegatonDepState::NotSetPath => {
            cu::warn!("'megaton' dependency is not set to local path");
            true
        }
        MegatonDepState::NotRightPath => {
            cu::warn!("'megaton' dependency is not set to the expected local path");
            true
        }
        MegatonDepState::Ok => false
    };
    if megaton_not_ok {
        cu::hint!("please add or update the dependency to `megaton.path = \"{expected_megaton_path}\"`");
    }

    Ok(RustProject { 
        has_cxx, has_build_script, output_path, include_namespace: crate_name.to_string() })
}

/// Run `cargo metadata` to check cxx version
pub fn check_cxx_version_with_cargo_metadata(manifest_path: &Path) -> cu::Result<()> {
    let metadata = cu::check!(MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec(), "failed to execute cargo-metadata")?;
    let cxx = match metadata.packages.iter().find(|pack| pack.name == "cxx") {
        Some(package) => package,
        None => {
            cu::debug!("cxx package not found, skipping cxx version check");
            return Ok(());
        }
    };
    check_cxx_version(&cxx.version)
}

#[cu::context("failed to validate 'cxx' dependency")]
fn check_cxx_version(current_version: &Version) -> cu::Result<()> {
    let blessed_version = cxxbridge::blessed_semver();
    if current_version < &blessed_version {
        cu::bail!(
            "cxx version is older than the supported version; supported: {}, found: {} ",
            blessed_version,
            current_version
        );
    } else if current_version > &blessed_version {
        cu::warn!(
            "cxx version is newer than the supported version; supported: {}, found: {}",
            blessed_version,
            current_version
        );
    } else {
        cu::debug!("cxx version is equal to the blessed version");
    };
    Ok(())
}

/// Just run the build script for the project, since the build script may generate
/// additional files needed for later steps
#[cu::context("failed to run cargo build script")]
pub async fn run_build_script(
    config: &Config, 
    toolchain: &ToolchainEnv,
    rust_env: &RustEnv,
) -> cu::Result<()> {
    // cargo doesn't have a way to just run the build script, the best option is to run check
    run_cargo("running cargo build script", config, toolchain, rust_env, ["check"]).await
}

/// Build the full crate
#[cu::context("failed to build rust crate")]
pub async fn run_build_full(
    config: &Config, 
    toolchain: &ToolchainEnv,
    rust_env: &RustEnv,
) -> cu::Result<()> {
    run_cargo("running cargo build", config, toolchain, rust_env, ["build"]).await
}

/// Run cargo command in the current project
async fn run_cargo<C: IntoIterator<Item=impl AsRef<OsStr>>>(
    desc: &str,
    config: &Config, 
    toolchain: &ToolchainEnv,
    rust_env: &RustEnv,
    command: C
) -> cu::Result<()> {
    let build_config = config.build_config()?;
    let rustflags = &build_config.flags.resolved.rustflags;
    let cargo_flags = &build_config.flags.resolved.cargo;
    let c_flags = build_config.flags.resolved.c.join(" ");
    let cxx_flags = &build_config.flags.resolved.cxx.join(" ");
    // cargo doesn't have a way to just run the build script, the best option is to run check
    let (child, bar) = rust_env.host.cargo.command()
        .env("RUSTFLAGS", rustflags)
        .env("CRATE_CC_NO_DEFAULTS", "1") // stop cc crate from adding default flags
        .env("CC_aarch64-unknown-hermit", &toolchain.devkita64.cc)
        .env("CXX_aarch64-unknown-hermit", &toolchain.devkita64.cxx)
        .env("AR_aarch64-unknown-hermit", &toolchain.devkita64.ar)
        .env("CFLAGS_aarch64-unknown-hermit", &c_flags)
        .env("CXXFLAGS_aarch64-unknown-hermit", &cxx_flags)
        .add(cu::args![MEGATON_TOOLCHAIN_SELECTOR, "--manifest-path", &config.rust.cargo_manifest])
        .args(cargo_flags)
        .args(command)
        .preset(cu::pio::cargo(desc))
        .co_spawn().await?;
    child.co_wait_nz().await?;
    bar.done();
    Ok(())
}

/// Run cxxbridge over rust files that need to generate .cc and header files. Return a list
/// of source files that need to be compiled
#[cu::context("failed to generate cxxbridge files")]
pub async fn run_cxxbridge(config: &Config, toolchain: &ToolchainEnv, project: &RustProject) -> cu::Result<Vec<PathBuf>> {
    let pool = cu::co::pool(-1);
    let header_suffix = &config.rust.header_suffix;
    let header_extension = if header_suffix .is_empty() {
        None
    } else {Some(format!(".rs{header_suffix}"))};
    let inc_root = config.project.out_inc.join(&project.include_namespace);
    let mut handles = vec![];
    let source_root = &config.rust.source;
    let mut walker = cu::fs::walker(source_root);
    walker.follow_links(true);
    let walk = 
    cu::check!(walker.walk(), "failed to process rust source directory with cxxbridge: {}", source_root.display())?;

    let bar = cu::progress("running cxxbridge").total(0).percentage(false).eta(false)
        .spawn();

    for entry in walk {
        let Ok(entry) = entry else {
            cu::debug!("ignoring entry with error");
            continue;
        };
        if !entry.file_name().is_some_and(|x| x.as_encoded_bytes().ends_with(b".rs")) {
            continue;
        }
        let out_inc_path = {
            let mut inc_path = entry.rel_path()?;
            if let Some(x) = &header_extension {
                inc_path.set_extension(x);
            }
            inc_root.join(inc_path)
        };
        let src_cc_path = path_hash::hashed_output_path(entry.path(), &config.project.out_src)?;
        let input_path = entry.path().to_path_buf();

        let cxxbridge = toolchain.cxxbridge.path.clone();
        let bar = bar.clone();


        let handle = pool.spawn(async move {
            let (child, stdout, stderr) = cxxbridge.command()
            .add(cu::args![&input_path])
                .stdout(cu::pio::string())
                .stderr(cu::pio::string())
                .stdin_null()
                .co_spawn().await?;
            let result = child.co_wait_nz().await;
            if let Err(e) = result{
                let stderr = stderr.co_join().await??;
                if stderr.starts_with("cxxbridge: no #[cxx::bridge] module found") {
                    cu::progress!(bar += 1);
                    return cu::Ok(None);
                }
                cu::rethrow!(e, "cxxbridge failed to generate source: {stderr}");
            }
            let stdout = stdout.co_join().await??;
            cu::fs::write(&src_cc_path, stdout)?;

            let (child, stdout, stderr) = cxxbridge.command()
            .add(cu::args![&input_path, "--header"])
                .stdout(cu::pio::string())
                .stderr(cu::pio::string())
                .stdin_null()
                .co_spawn().await?;
            let result = child.co_wait_nz().await;
            if let Err(e) = result{
                let stderr = stderr.co_join().await??;
                cu::rethrow!(e, "cxxbridge failed to generate header: {stderr}");
            }
            let stdout = stdout.co_join().await??;
            cu::fs::write(&out_inc_path, stdout)?;

            cu::progress!(bar += 1, "processed {}", src_cc_path.try_to_rel().display());


            Ok(Some(src_cc_path))
        });

        handles.push(handle);
    }

    if handles.is_empty() {
        bar.done_with_message("no rust file found");
        return Ok(vec![]);
    }

    bar.set_total(handles.len() as u64);

    let mut set = cu::co::set(handles);
    let mut error_count = 0;
    let mut src_cc_paths = vec![];
    while let Some(result) = set.next().await {
        let result = result?;
        match result {
            Err(e) => {
                cu::error!("{e:?}");
                error_count += 1;
            }
            Ok(None) => { }
            Ok(Some(x)) => {
                src_cc_paths.push(x)
            }
        }
    }
    if error_count > 0 {
        cu::bail!("{error_count} errors while running cxxbridge");
    }
    bar.done_with_message(&format!("processed {} files with cxxbridge attributes", src_cc_paths.len()));

    Ok(src_cc_paths)
}
