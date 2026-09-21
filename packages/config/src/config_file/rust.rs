use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::config_file::{CaptureUnused, ProjectTargetEnv, Validate, ValidateCtx};

/// `[rust]` config section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RustConfig {
    /// If rust is enabled. Default is `true` if `cargo_manifest` is found
    /// or is not "Cargo.toml"
    enabled: Option<bool>,

    #[serde(default = "RustConfig::default_manifest_path")]
    pub cargo_manifest: PathBuf,

    #[serde(default = "RustConfig::default_header_suffix")]
    pub header_suffix: String,

    #[serde(default = "RustConfig::default_sources")]
    pub sources: Vec<PathBuf>,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            cargo_manifest: Self::default_manifest_path(),
            header_suffix: Self::default_header_suffix(),
            sources: Self::default_sources(),
            enabled: Default::default(),
            unused: Default::default(),
        }
    }
}

impl RustConfig {
    pub fn resolve(&mut self, project: &ProjectTargetEnv) {
        let resolved_manifest = project.root.join(&self.cargo_manifest);
        match self.enabled {
            None => {
                if self.cargo_manifest.as_path() != Path::new("Cargo.toml") {
                    cu::debug!("rust is enabled because rust.cargo-manifest is specified");
                    self.enabled = Some(true);
                } else if resolved_manifest.exists() {
                    cu::debug!("rust is enabled because cargo manifest is found");
                    self.enabled = Some(true);
                } else {
                    cu::debug!("rust is not enabled because cargo manifest is not found");
                }
            }
            Some(true) => {
                cu::debug!("rust is enabled in config");
            }
            Some(false) => {
                cu::debug!("rust is disabled in config");
            }
        }
        self.cargo_manifest = resolved_manifest;
        for s in &mut self.sources {
            *s = project.root.join(&*s);
        }
    }
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or_default()
    }
    fn default_manifest_path() -> PathBuf {
        PathBuf::from("Cargo.toml")
    }

fn default_header_suffix() -> String {
    String::from(".h")
}

fn default_sources() -> Vec<PathBuf> {
    vec![PathBuf::from("src")]
}
    }

impl Validate for RustConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}
