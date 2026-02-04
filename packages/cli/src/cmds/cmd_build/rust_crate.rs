use std::path::{Path, PathBuf};

use cu::pre::*;

use super::config::CargoConfig;

pub struct RustCrate {
    pub manifest: PathBuf,
    target_path: PathBuf, // Not necessarily the same as the Megaton target dir
    source_paths: Vec<PathBuf>,
}

impl RustCrate {
    /// Gets the crate based on the cargo config. Returns `Ok(None)`. Errors if
    /// cargo is explicitly enabled, but couldn't be be found for some reason.
    pub fn from_config(cargo: CargoConfig) -> cu::Result<Option<Self>> {
        let manifest = cargo
            .manifest
            .unwrap_or(CargoConfig::default_manifest_path());

        match cargo.enabled {
            None => Ok(RustCrate::new(&manifest, cargo.sources).ok()),
            Some(true) => Ok(Some(
                RustCrate::new(&manifest, cargo.sources)
                    .context("Cargo enabled, but failed to find the crate")?,
            )),
            Some(false) => Ok(None),
        }
    }

    /// Prefer to use `from_config(...)` when possible
    fn new(manifest_path: &Path, sources: Vec<PathBuf>) -> cu::Result<Self> {
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
        })
    }

    /// Build the rust crate with `cargo build +megaton`
    pub async fn build(&self, cargoflags: &[String], rustflags: &str) -> cu::Result<bool> {
        cu::info!("Building rust crate!");
        let cargo = cu::which("cargo").context("Cargo executable not found")?;

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

        command = command.args(cargoflags);
        command = command.env("RUSTFLAGS", rustflags);

        let child = command.co_spawn().await?;
        child.co_wait_nz().await.context("Cargo build failed")?;

        // TODO: return false if cargo did nothing
        Ok(true)
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

    /// Gets the path to the static lib compiled by cargo
    /// This should always be using the hermit target on release mode
    /// Will panic if it fails to read the package name from Cargo.toml
    pub fn get_output_path(&self) -> cu::Result<PathBuf> {
        let rel_path = self
            .target_path
            .join("aarch64-unknown-hermit")
            .join("release");
        let name = &cu::fs::read_string(&self.manifest)
            .unwrap()
            .parse::<toml::Table>()
            .unwrap();

        let name = &name["package"]["name"].as_str().unwrap();
        let name = name.replace("-", "_");
        let filename = format!("lib{name}.a");
        let path = rel_path
            .join(&filename)
            .canonicalize()
            .context(format!("{} does not exist", &filename))?;

        Ok(path)
    }


    /// Scan rust sources and generate cxxbridge sources and headers
    pub async fn gen_cxxbridge(&self) -> cu::Result<()> {
        Ok(())
    }
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
