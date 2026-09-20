use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::config_file::{self, Validate, ValidateCtx, CaptureUnused, ExtendProfile};

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

    /// C/C++ Defines in the form of `K=V`
    #[serde(default)]
    pub defines: Vec<String>,

    /// Additional Linker scripts
    #[serde(default)]
    pub ldscripts: Vec<PathBuf>,

    /// Additional objects to link
    #[serde(default)]
    pub objects: Vec<PathBuf>,

    #[serde(default)]
    pub compiler: BuildCompilerConfig,

    #[serde(default)]
    pub flags: BuildFlagConfig,

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
        config_file::extend_by_appending(&mut self.sources, &other.sources);
        config_file::extend_by_appending(&mut self.includes, &other.includes);
        config_file::extend_by_appending(&mut self.defines, &other.defines);
        config_file::extend_by_appending(&mut self.ldscripts, &other.ldscripts);
        config_file::extend_by_appending(&mut self.objects, &other.objects);

        self.compiler.extend_profile(&other.compiler);
        self.flags.extend_profile(&other.flags);
    }
}

/// `[build.compiler]` config section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildCompilerConfig {
    pub common: Option<Vec<String>>,
    pub cc: Option<Vec<String>>,
    pub cxx: Option<Vec<String>>,
    #[serde(rename = "as")]
    pub as_: Option<Vec<String>>,
    pub ar: Option<Vec<String>>,
    pub ld: Option<Vec<String>>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl ExtendProfile for BuildCompilerConfig {
    fn extend_profile(&mut self, other: &Self) {
        extend_flags(&mut self.common, &other.common);
        extend_flags(&mut self.c, &other.c);
        extend_flags(&mut self.cxx, &other.cxx);
        extend_flags(&mut self.as_, &other.as_);
        extend_flags(&mut self.ld, &other.ld);
    }
}


impl Validate for BuildCompilerConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}

/// `[build.flags]` config section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildFlagConfig {
    pub common: Option<Vec<String>>,
    pub c: Option<Vec<String>>,
    pub cxx: Option<Vec<String>>,
    #[serde(rename = "as")]
    pub as_: Option<Vec<String>>,
    pub ld: Option<Vec<String>>,
    pub rust: Option<Vec<String>>,
    pub cargo: Option<Vec<String>>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl ExtendProfile for BuildFlagConfig {
    fn extend_profile(&mut self, other: &Self) {
        extend_flags(&mut self.common, &other.common);
        extend_flags(&mut self.c, &other.c);
        extend_flags(&mut self.cxx, &other.cxx);
        extend_flags(&mut self.as_, &other.as_);
        extend_flags(&mut self.ld, &other.ld);
    }
}


impl Validate for BuildFlagConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}
