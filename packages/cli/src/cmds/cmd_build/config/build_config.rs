// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use super::{CaptureUnused, ExtendProfile, FlagConfig, Validate, ValidateCtx};
use cu::pre::*;

/// Config in the `[build]` section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Build {
    /// C/C++ Source directories, relative to Megaton.toml
    #[serde(default)]
    pub sources: Vec<String>,

    /// C/C++ Include directories, relative to Megaton.toml
    #[serde(default)]
    pub includes: Vec<String>,

    /// Additional Library paths
    #[serde(default)]
    pub libpaths: Vec<String>,

    /// Additional Libraries to link with
    #[serde(default)]
    pub libraries: Vec<String>,

    /// Additional Linker scripts
    #[serde(default)]
    pub ldscripts: Vec<String>,

    /// Additional objects to link
    #[serde(default)]
    pub objects: Vec<String>,

    #[serde(default)]
    pub flags: FlagConfig,

    // TODO: add cargo flags
    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Validate for Build {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.flags.validate_property(ctx, "flags")?;
        cu::hint!("TODO: validate build.cargo");
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
