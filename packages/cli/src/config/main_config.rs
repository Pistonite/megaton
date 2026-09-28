// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

//! Config structures
use std::path::{Path, PathBuf};

use cu::pre::*;
use semver::VersionReq;

use crate::config::util;

use super::{BASE_PROFILE, Build, CaptureUnused, ExtendProfile, Profile, Validate, ValidateCtx};

/// Get the root path of the project
pub fn get_root_and_manifest(manifest_path: Option<&str>) -> cu::Result<(PathBuf, PathBuf)> {
    if let Some(p) = manifest_path {
        let manifest_path = Path::new(p).normalize()?;
        let root = manifest_path.parent_abs()?;
        return Ok((root, manifest_path));
    }
    let cwd = PathBuf::from(".").normalize()?;
    for path in cwd.ancestors() {
        let manifest_path = PathBuf::from(path).join("Megaton.toml").normalize();
        let Ok(manifest_path) = manifest_path else {
            continue;
        };
        if !manifest_path.exists() {
            continue;
        }
        let root = path.normalize()?;
        return Ok((root, manifest_path));
    }
    cu::bail!("failed to resolve project root; please ensure Megaton.toml exists");
}

/// Load a Megaton.toml config file
pub fn load(path: &Path) -> cu::Result<Config> {
    let content = cu::fs::read_string(path)?;
    let config = cu::check!(
        toml::parse::<Config>(&content),
        "failed to parse Megaton config"
    )?;
    config.validate_root()?;
    Ok(config)
}

/// Config data read from Megaton.toml
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The `[module]` section
    ///
    /// Basic module info
    pub module: Module,

    /// The `[profile]` section
    ///
    /// Configure behavior when selecting profile from command line
    #[serde(default)]
    pub profile: ProfileConfig,

    /// The `[megaton]` section
    ///
    /// Metadata for megaton build tool
    #[serde(default)]
    pub megaton: MegatonConfig,

    /// The `[cargo]` section
    ///
    /// Specify customizations for crates
    #[serde(default)]
    pub cargo: CargoConfig,

    /// The `[build]` section
    ///
    /// Specify options for building C/C++/assembly project code
    /// and linking the module
    pub build: Profile<Build>,

    /// The `[check]` section (for checking unresolved dynamic symbols)
    pub check: Option<Profile<Check>>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Validate for Config {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.module.validate_property(ctx, "module")?;
        self.profile.validate_property(ctx, "profile")?;
        self.megaton.validate_property(ctx, "megaton")?;
        self.cargo.validate_property(ctx, "cargo")?;
        self.build.validate_property(ctx, "build")?;

        if let Some(check) = &self.check {
            check.validate_property(ctx, "check")?;
        }

        if !self.megaton.lib_enabled() && self.cargo.enabled.is_some_and(|val| val) {
            cu::bail!("rust cannot be enabled unless libmegaton is enabled");
        }

        self.unused.validate(ctx)
    }
}




