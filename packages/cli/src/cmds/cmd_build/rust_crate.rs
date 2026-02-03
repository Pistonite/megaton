use std::path::{Path, PathBuf};

use cu::pre::*;

use super::config::Flags;

// A rust crate that will be built as a component of the megaton lib or the mod
struct RustCrate {
    manifest: PathBuf,
    got_built: bool,
}

impl RustCrate {
    pub fn new(manifest_path: &Path) -> cu::Result<Self> {
        let manifest = manifest_path.to_owned().canonicalize().context(format!(
            "Could not find Cargo.toml at {:?}",
            manifest_path.display()
        ))?;

        Ok(Self {
            manifest,
            got_built: false,
        })
    }

    pub async fn build(&mut self, cargoflags: &[String], rustflags: &str) -> cu::Result<bool> {
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

        let exit_code = command.co_spawn().await?.co_wait_nz();
        if !exit_code.success() {
            return Err(cu::Error::msg(format!(
                "Cargo build failed with exit status {:?}",
                exit_code
            )));
        }

        self.got_built = true;
        Ok(self.got_built)
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
        // assuming cargo is in release mode
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
