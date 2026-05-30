// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::path::PathBuf;

use cu::pre::*;

use super::{CaptureUnused, ExtendProfile, FlagConfig, Validate, ValidateCtx};

/// Config in the `[build]` section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Build {
    /// C/C++ Source directories, relative to Megaton.toml
    #[serde(default)]
    pub sources: Vec<PathBuf>,

    /// C/C++ Include directories, relative to Megaton.toml
    #[serde(default)]
    pub includes: Vec<PathBuf>,

    /// Additional Library paths
    #[serde(default)]
    pub libpaths: Vec<PathBuf>,

    /// Additional Libraries to link with
    #[serde(default)]
    pub libraries: Vec<String>,

    /// Additional Linker scripts
    #[serde(default)]
    pub ldscripts: Vec<PathBuf>,

    /// Additional objects to link
    #[serde(default)]
    pub objects: Vec<PathBuf>,

    #[serde(default)]
    pub flags: FlagConfig,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Validate for Build {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.flags.validate_property(ctx, "flags")?;
        self.unused.validate(ctx)?;
        Ok(())
    }
}

impl ExtendProfile for Build {
    fn extend_profile(&mut self, other: &Self) {
        self.sources.extend(other.sources.iter().cloned());
        self.includes.extend(other.includes.iter().cloned());
        self.libpaths.extend(other.libpaths.iter().cloned());
        self.libraries.extend(other.libraries.iter().cloned());
        self.ldscripts.extend(other.ldscripts.iter().cloned());
        self.objects.extend(other.objects.iter().cloned());
        self.flags.extend_profile(&other.flags);
    }
}
