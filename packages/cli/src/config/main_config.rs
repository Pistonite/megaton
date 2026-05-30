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
    cu::bail!("failed to determine root of project; please ensure Megaton.toml exists");
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

/// The `[cargo]` section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CargoConfig {
    pub enabled: Option<bool>,

    pub manifest: Option<PathBuf>,

    #[serde(default = "default_header_suffix")]
    pub header_suffix: String,

    #[serde(default = "default_sources")]
    pub sources: Vec<PathBuf>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Default for CargoConfig {
    fn default() -> Self {
        Self {
            enabled: None,
            manifest: None,
            header_suffix: default_header_suffix(),
            sources: default_sources(),
            unused: Default::default(),
        }
    }
}

impl CargoConfig {
    pub fn default_manifest_path() -> PathBuf {
        PathBuf::from("Cargo.toml")
    }
}

fn default_header_suffix() -> String {
    String::from(".h")
}

fn default_sources() -> Vec<PathBuf> {
    vec![PathBuf::from("src")]
}

impl Validate for CargoConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}

/// The `[megaton]` section
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MegatonConfig {
    /// If specified, megaton will run version check and error if the current
    /// version doesn't match the given requirement. To only check minor version,
    /// pass a version of the form `0.{minor}.*`
    pub version: Option<VersionReq>,

    /// Custom entry point symbol for the module. This should only be set if libmegaton is disabled.
    /// Using this disables the libmegaton runtime, which also means rust support will be disabled.
    pub custom_entry: Option<String>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Validate for MegatonConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        if let Some(v) = &self.version {
            util::check_megaton_version_requirement(v)?
        }
        self.unused.validate(ctx)
    }
}

impl MegatonConfig {
    /// Get the entry point symbol name
    pub fn entry_point(&self) -> &str {
        match &self.custom_entry {
            None => "__megaton_module_entry",
            Some(entry) => {
                if entry.is_empty() {
                    "__megaton_module_entry"
                } else {
                    entry
                }
            }
        }
    }

    /// Checks if libmegaton enabled
    pub fn lib_enabled(&self) -> bool {
        self.custom_entry.clone().is_none_or(|val| val.is_empty())
    }
}

/// Config in the `[module]` section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Module {
    /// The name of the module, used as the target name of the final binary.
    pub name: String,
    /// The title ID as a 64-bit integer, used for generating the npdm file.
    pub title_id: u64,

    /// The target directory to put build files
    #[serde(default = "Module::default_target")]
    target: PathBuf,

    /// The compile_commands.json file to put/update compile commands
    /// for other tools like clangd
    #[serde(default = "Module::default_comp_commands")]
    compdb: PathBuf,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Validate for Module {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        if self.name.is_empty() {
            cu::bail!("module.name must be non-empty");
        }
        if self.name == "lib" {
            cu::bail!("'lib' is reserved and cannot be the module name");
        }
        if self
            .name
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
        {
            cu::bail!(
                "'{}' is not a valid module name (must only contain alphanumeric characters, - or _)",
                self.name
            );
        }
        self.unused.validate(ctx)
    }
}

impl Module {
    /// Get the title ID as a lower-case hex string (without the `0x` prefix)
    pub fn title_id_hex(&self) -> String {
        format!("{:016x}", self.title_id)
    }
    pub fn target_path(&self, root: &Path) -> PathBuf {
        root.join(&self.target)
    }
    fn default_target() -> PathBuf {
        "target".into()
    }
    pub fn compdb_path(&self, root: &Path) -> PathBuf {
        root.join(&self.compdb)
    }
    fn default_comp_commands() -> PathBuf {
        "compile_commands.json".into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Set the profile to use when profile is unspecified
    ///
    /// If `Some("")`, a profile must be specified in command line or megaton will error
    pub default: Option<String>,

    /// Allow the base (`none`) profile to be used
    #[serde(default = "default_true")]
    pub allow_base: bool,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            default: None,
            allow_base: true,
            unused: Default::default(),
        }
    }
}

impl Validate for ProfileConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}

impl ProfileConfig {
    /// Select profile based on command line and config
    ///
    /// Prints formatted error message on failure
    pub fn resolve<'a, 'b>(&'a self, cli_profile: &'b str) -> cu::Result<&'a str>
    where
        'b: 'a, // lifetime: cli should live longer since that's parsed before config
    {
        let profile = match (cli_profile, &self.default) {
            ("none", Some(p)) if p.is_empty() => {
                cu::error!("no profile specified!");
                cu::hint!("- please specify a profile with -p PROFILE");
                cu::bail!("failed to selected a profile");
            }
            ("none", Some(p)) => p,
            ("none", None) => BASE_PROFILE,
            (profile, _) => profile,
        };

        if !self.allow_base && profile == "none" {
            cu::error!("base profile is not allowed!");
            cu::hint!("- please specify a profile with -p PROFILE");
            cu::bail!("failed to selected a profile");
        }

        Ok(profile)
    }
}

/// The `check` section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Check {
    /// Symbols to ignore
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Paths to *.syms file (output of objdump) that contains dynamic symbols accessible by the module
    #[serde(default)]
    pub symbols: Vec<PathBuf>,
    /// Extra instructions to disallow (like `"msr"`). Values are regular expressions.
    #[serde(default)]
    pub disallowed_instructions: Vec<String>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Validate for Check {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}

impl ExtendProfile for Check {
    fn extend_profile(&mut self, other: &Self) {
        self.ignore.extend(other.ignore.iter().cloned());
        self.symbols.extend(other.symbols.iter().cloned());
        self.disallowed_instructions
            .extend(other.disallowed_instructions.iter().cloned());
    }
}

fn default_true() -> bool {
    true
}
