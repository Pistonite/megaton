use std::path::{Path, PathBuf};

use cu::pre::*;

use super::config::CargoConfig;

pub struct RustCtx {
    pub manifest: PathBuf,
    target_path: PathBuf, // Not necessarily the same as the Megaton target dir
    source_paths: Vec<PathBuf>,
    header_suffix: String,
}

impl RustCtx {
    /// Gets the crate based on the cargo config. Returns `None` if rust is
    /// disabled or can't be automattically enabled. Returns Some(Err()) if
    /// cargo is explicitly enabled, but couldn't be be found for some reason.
    pub fn from_config(cargo: CargoConfig) -> Option<cu::Result<Self>> {
        let manifest = cargo
            .manifest
            .unwrap_or(CargoConfig::default_manifest_path());

        // Nested enums is not ideal. Maybe try and find a better way to do this while maintaing
        // a return type that makes sense for the caller.
        match cargo.enabled {
            None | Some(true) => {
                let ctx = RustCtx::new(&manifest, cargo.sources, cargo.header_suffix);
                if ctx.is_err() && cargo.enabled.is_none() {
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

    /// Build the rust crate with `cargo build +megaton`
    pub async fn build(&self, cargoflags: &[String], rustflags: &str) -> cu::Result<bool> {
        let old_output = self.get_output_path();
        let old_mtime = match old_output {
            Some(file) => cu::fs::get_mtime(file).unwrap_or(None),
            None => None,
        };

        let cargo = cu::which("cargo").context("Cargo executable not found")?;
        let mut command = cargo
            .command()
            .add(cu::args![
                "+megaton",
                "build",
                "--manifest-path",
                &self.manifest,
            ])
            .preset(cu::pio::cargo("Updating crate"));

        command = command.args(cargoflags);
        command = command.env("RUSTFLAGS", rustflags);

        command.co_spawn().await?.0.co_wait_nz().await?;

        let new_output = self.get_output_path().unwrap();
        let new_mtime = cu::fs::get_mtime(new_output).unwrap();

        // Return true if artifact changed
        Ok(new_mtime != old_mtime)
    }

    fn get_source_files(&self) -> cu::Result<Vec<PathBuf>> {
        // TODO: try to make this return an iterator
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
    pub fn get_output_path(&self) -> Option<PathBuf> {
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

        rel_path.join(&filename).canonicalize().ok()
    }

    /// Scan rust sources and generate cxxbridge sources and headers
    pub async fn gen_cxxbridge(&self) -> cu::Result<()> {
        cu::debug!("header suffix={}", &self.header_suffix);
        for src in &self.get_source_files()? {
            cu::debug!("rust source to scan={}", src.display())
        }
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
