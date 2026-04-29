// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use cargo_metadata::{MetadataCommand, semver::Version};
use cu::pre::*;

use crate::env::environment;

use super::config::CargoConfig;

#[derive(Debug, Clone)]
pub struct RustCtx {
    pub manifest: PathBuf,
    target_path: PathBuf,
    source_paths: Vec<PathBuf>,
    header_suffix: String,
}

static BLESSED_CXX_VERSION: &str = "1.0.194";

impl RustCtx {
    /// Gets the crate based on the cargo config. Returns `None` if rust is
    /// disabled or can't be automatically enabled. Returns Some(Err()) if
    /// cargo is explicitly enabled, but couldn't be be found for some reason.
    pub fn from_config(cargo: CargoConfig) -> Option<cu::Result<Self>> {
        let manifest = cargo
            .manifest
            .unwrap_or(CargoConfig::default_manifest_path());

        // Nested enums is not ideal. Maybe try and find a better way to do this while maintain
        // a return type that makes sense for the caller.
        match cargo.enabled {
            None | Some(true) => {
                let ctx = RustCtx::new(&manifest, cargo.sources, cargo.header_suffix);
                if ctx.is_err() && cargo.enabled.is_none() {
                    cu::hint!("Cargo is disabled");
                    None
                } else {
                    Some(ctx)
                }
            }
            Some(false) => None,
        }
    }

    // Not public since callers should use `from_config()` instead
    fn new(manifest_path: &Path, sources: Vec<PathBuf>, header_suffix: String) -> cu::Result<Self> {
        let manifest = manifest_path.to_owned().normalize().context(format!(
            "Could not find Cargo.toml at {:?}",
            manifest_path.display()
        ))?;

        let crate_root = manifest.parent().unwrap();

        cu::debug!("checking cxx version");

        let source_paths = sources
            .iter()
            .map(|rel_path| crate_root.join(rel_path))
            .collect::<Vec<_>>();

        // This should always be target, even if the megaton target dir is different
        let target_path = crate_root.join("target");

        Ok(Self {
            manifest,
            target_path,
            source_paths,
            header_suffix,
        })
    }

    pub fn check_cxx_version(&self) -> cu::Result<()> {
        let metadata = MetadataCommand::new()
            .manifest_path(&self.manifest)
            .exec()?;

        let cxx = match metadata.packages.iter().find(|pack| pack.name == "cxx") {
            Some(package) => package,
            None => {
                cu::debug!("cxx package not found, skipping cxx version check");
                return Ok(());
            }
        };

        let blessed_version = Version::parse(BLESSED_CXX_VERSION).unwrap();
        if cxx.version < blessed_version {
            cu::bail!(
                "cxx version is older than the supported version; supported: {}, found: {} ",
                blessed_version,
                cxx.version
            );
        }
        if cxx.version > blessed_version {
            cu::warn!(
                "cxx version is newer than the supported version; supported: {}, found: {}",
                blessed_version,
                cxx.version
            );
        }

        Ok(())
    }

    pub fn has_build_script(&self) -> bool {
        let script = self.manifest.parent().unwrap().join("build.rs");
        script.exists()
    }

    /// Add the megaton rust library to the crate
    /// libmegaton must be installed when this is called
    pub async fn add_megaton_rust_lib(&self, lib_path: &Path) -> cu::Result<()> {
        let cargo = cu::which("cargo")
            .context("Cargo executable not found: ensure rust is properly installed")?;
        let command = cargo
            .command()
            .add(cu::args![
                "add",
                "--manifest-path",
                &self.manifest,
                "--path",
                lib_path,
            ])
            .stdio_null()
            .stderr(cu::lv::I);

        command.co_wait_nz().await
    }

    /// Build the rust crate with `cargo build +megaton`
    pub async fn build(
        &self,
        cargoflags: &[String],
        rustflags: &str,
        check: bool,
    ) -> cu::Result<bool> {
        let old_output = self.get_output();
        let old_mtime = match old_output {
            Ok(file) => cu::fs::get_mtime(file).unwrap_or(None),
            Err(_) => None,
        };

        let cargo = cu::which("cargo")
            .context("Cargo executable not found: ensure rust is properly installed")?;
        let command = if check { "check" } else { "build" };
        let mut command = cargo
            .command()
            .add(cu::args![
                "+megaton",
                command,
                "--manifest-path",
                &self.manifest,
            ])
            .preset(cu::pio::cargo(if check {
                "Check rust crate"
            } else {
                "Build rust crate"
            }));

        command = command.args(cargoflags);
        command = command.env("RUSTFLAGS", rustflags);
        command = command.env("CC", environment().cc());
        command = command.env("CXX", environment().cxx());
        command = command.env("AR", environment().ar());

        command.co_spawn().await?.0.co_wait_nz().await?;
        if check {
            return Ok(false);
        }

        let new_output = self.get_output().unwrap();
        let new_mtime = cu::fs::get_mtime(new_output).unwrap();

        // Return true if artifact changed
        Ok(new_mtime != old_mtime)
    }

    /// Gets the path to the static lib compiled by cargo
    /// Error if the Cargo manifest can't be parsed, filename
    /// can't be parsed, or if file doesn't exist
    pub fn get_output(&self) -> cu::Result<PathBuf> {
        let rel_path = self
            .target_path
            .join("aarch64-unknown-hermit")
            .join("release");

        let table = &cu::fs::read_string(&self.manifest)?.parse::<toml::Table>()?;

        let package = &table.get("package");
        let package = cu::check!(package, "cargo manifest missing entry package")?;
        let name = &package.get("name");
        let name = cu::check!(name, "cargo manifest missing entry package.name")?;
        let name = cu::check!(name.as_str(), "package name is not a string")?;
        let name = name.replace("-", "_");

        let filename = format!("lib{name}.a");
        rel_path.join(&filename).normalize_exists()
    }

    /// Scan rust sources and generate cxxbridge sources and headers
    /// Returns Ok(true) if any file was generated/changed
    pub async fn gen_cxxbridge(
        self,
        src_out_path: &Path,
        include_out_path: &Path,
    ) -> cu::Result<bool> {
        let mut something_changed =
            if cxxbridge_cmd(None, true, &include_out_path.join("rust").join("cxx.h")).await? {
                cu::debug!("Cxxbridge: generated rust/cxx.h");
                true
            } else {
                cu::debug!("Cxxbridge: header up to date rust/cxx.h");
                false
            };

        let suffix = Arc::new(self.header_suffix.clone());
        let source_paths = Arc::new(self.source_paths.clone());
        let src_out_path = Arc::new(src_out_path.to_owned());
        let include_out_path = Arc::new(include_out_path.to_owned());
        let pool = cu::co::pool(0);
        let mut handles = vec![];
        for file in self.get_source_files()? {
            handles.push(pool.spawn(cxxbridge_process(
                file,
                src_out_path.clone(),
                include_out_path.clone(),
                suffix.clone(),
                source_paths.clone(),
            )));
        }
        let mut set = cu::co::set(handles);

        let mut errors = vec![];
        while let Some(joined) = set.next().await {
            let res = joined.context("Failed to join handle")?;
            match res {
                Ok(updated) => {
                    something_changed |= updated;
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            let num = errors.len();
            let errorstring = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            cu::bail!("Failed due to {num} error(s): \n{errorstring}");
        }
        Ok(something_changed)
    }

    fn get_source_files(&self) -> cu::Result<Vec<PathBuf>> {
        let mut source_files: Vec<PathBuf> = vec![];
        for dir in &self.source_paths {
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

async fn cxxbridge_process(
    file: PathBuf,
    src_out_path: Arc<PathBuf>,
    include_out_path: Arc<PathBuf>,
    header_suffix: Arc<String>,
    source_paths: Arc<Vec<PathBuf>>,
) -> cu::Result<bool> {
    let stem_os = cu::check!(file.file_stem(), "Invalid file name: {}", file.display())?;

    let stem = stem_os.as_utf8()?;
    let path_to_rs = file.normalize()?;

    let rel_source_path = source_paths
        .iter()
        .find(|p| path_to_rs.starts_with(p))
        .unwrap();
    let rel_source_path = path_to_rs.strip_prefix(rel_source_path)?;
    let mut out_h = include_out_path.join(rel_source_path);
    let mut out_cc = src_out_path.join(rel_source_path);
    out_h.set_file_name(format!("{stem}{header_suffix}"));
    out_cc.set_file_name(format!("{stem}.cc"));

    let header_updated = if cxxbridge_cmd(Some(&file), true, &out_h).await? {
        cu::debug!("Cxxbridge: generated header {}", &out_h.display());
        true
    } else {
        cu::debug!("Cxxbridge: header up to date {}", &out_h.display());
        false
    };

    let source_updated = if cxxbridge_cmd(Some(&file), false, &out_cc).await? {
        cu::debug!("Cxxbridge: generated source {}", &out_cc.display());
        true
    } else {
        cu::debug!("Cxxbridge: source up to date {}", &out_cc.display());
        false
    };

    Ok(header_updated || source_updated)
}

// Run the cxxbridge cmd and update the corresponding file if changed
// returns Ok(true) iff new code was generated and written
async fn cxxbridge_cmd(file: Option<&Path>, header: bool, output: &Path) -> cu::Result<bool> {
    let env = environment();
    let mut args = vec![];
    if let Some(file) = file {
        args.push(cu::check!(file.to_str(), "Not utf-8: {}", file.display())?);
    }
    if header {
        args.push("--header");
    }

    let exe = env.cxxbridge();
    if !exe.exists() {
        cu::debug!("cxxbridge-cmd not installed, installing to MEGATON_HOME/bin");
        install_cxxbridge(env.home())
            .await
            .context("Failed to install cxxbridge Make sure you are connected to the internet")?;
    }

    let command = exe
        .command()
        .stdout(cu::pio::buffer())
        .stderr(cu::pio::string())
        .stdin_null()
        .args(args);

    let (child, stdout, stderr) = command
        .co_spawn()
        .await
        .context("Failed to spawn cxxbridge")?;
    let status = child.co_wait().await.context("Failed to wait cxxbridge")?;

    match status.code() {
        Some(0) => {
            let stdout = stdout.co_join().await??;
            write_if_changed(output, &stdout)
        }
        Some(1) => Ok(false),
        Some(other) => {
            let stderr = stderr.co_join().await??;
            cu::bail!("cxxbridge exited with unexpected status ({other})\n{stderr}");
        }
        None => cu::bail!("cxxbridge terminated early"),
    }
}

async fn install_cxxbridge(dir: &Path) -> cu::Result<()> {
    cu::fs::make_dir(dir)?;
    let dir = dir.as_utf8()?;
    let command = cu::which("cargo")?
        .command()
        .add(cu::args![
            "install",
            "--root",
            dir,
            "--no-track",
            "-q", // suppress warning about adding ./bin to path
            format!("cxxbridge-cmd@{BLESSED_CXX_VERSION}"),
        ])
        .preset(cu::pio::cargo("Installing cxxbridge-cmd"));

    command.co_spawn().await?.0.co_wait_nz().await
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_new() {
//
//     }
// }
