use std::path::{Path, PathBuf};

use cu::pre::*;

#[derive(Debug, Serialize)]
pub struct HomeEnv {
    pub path: PathBuf,
}
impl HomeEnv {
    pub fn resolve() -> cu::Result<Self> {
        Ok(Self { path: get_megaton_home()? })
    }
    pub fn bin_path(&self, bin_name: &str) -> PathBuf {
        let mut p = self.path.join("bin");
        if cfg!(windows) {
            p.push(format!("{bin_name}.exe"));
        } else {
            p.push(bin_name);
        }
        p
    }
    pub fn rust_install_path(&self) -> PathBuf {
        self.path.join("rust-toolchain")
    }
    pub fn rust_source_path(&self) -> PathBuf {
        self.path.join("rust")
    }
}

#[cu::context("failed to resolve megaton home")]
fn get_megaton_home() -> cu::Result<PathBuf> {
    let megaton_home = cu::env_var("MEGATON_HOME").unwrap_or_default();
    let megaton_home = if megaton_home.is_empty() {
        cu::trace!("MEGATON_HOME not specified, using default path ~/.cache/megaton");
        let mut home = cu::check!(std::env::home_dir(), "failed to get user's home directory")?;
        home.extend([".cache", "megaton"]);
        home.normalize()?
    } else {
        let path = Path::new(&megaton_home).normalize()?;
        cu::trace!("Using MEGATON_HOME={}", path.display());
        path
    };
    Ok(megaton_home)
}
