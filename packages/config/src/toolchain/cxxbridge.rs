use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::toolchain::HomeEnv;

#[derive(Debug, Serialize)]
pub struct CxxBridgeEnv {
    pub path: PathBuf,
    pub version: String,
}

impl CxxBridgeEnv {
    pub fn resolve(home: &HomeEnv) -> Self {
        let binary = home.bin_path("cxxbridge");
        let version = get_cxxbridge_version(&binary).unwrap_or_default();
        Self {
            path: binary,
            version
        }
    }

    pub fn is_installed(&self) ->
     bool {
        !self.version.is_empty() && self.path.is_file()
    }
}

#[cu::context("failed to resolve cxxbridge version")]
fn get_cxxbridge_version(binary: &Path) -> cu::Result<String> {
        let (child, stdout) = binary
            .command()
            .arg("--version")
            .stdout(cu::pio::string())
            .stdie_null()
            .spawn()?;
    child.wait_nz()?;
    let version_string = stdout.join()??;
    let version = match version_string.strip_prefix("cxxbridge ") {
        Some(v) => v.trim(),
        None => {
            cu::error!("failed to parse cxxbridge version: {version_string}");
            cu::bail!("bad cxxbridge version")
        }
    };
    Ok(version.to_string())
}
