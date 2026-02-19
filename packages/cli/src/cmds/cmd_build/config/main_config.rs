// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

//! Config structures
use std::path::{Path, PathBuf};

use super::{BASE_PROFILE, Build, CaptureUnused, ExtendProfile, Profile, Validate, ValidateCtx};
use cu::pre::*;

/// Load a Megaton.toml config file
pub fn load_config(manifest_path: impl AsRef<Path>) -> cu::Result<Config> {
    let cwd = PathBuf::from(".").normalize().unwrap();
    let ancestors = cwd.ancestors();

    for path in ancestors {
        let p = PathBuf::from(path).join(&manifest_path).normalize();
        match p {
            Ok(p) => {
                if p.exists() {
                    std::env::set_current_dir(p.parent().unwrap())
                        .expect("Could not open megaton project root");
                    let content = cu::fs::read_string(p)?;
                    let config = toml::parse::<Config>(&content)
                        .context("Failed to parse Megaton config")?;
                    config.validate_root()?;
                    return Ok(config);
                }
            }
            Err(_) => continue,
        }
    }
    Err(cu::Error::msg("Failed to find Megaton config"))
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
        cu::hint!("TODO: validate cargo config");
        self.unused.validate(ctx)
    }
}

/// The `[megaton]` section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MegatonConfig {
    /// If specified, megaton will run version check
    ///
    /// `N` means the build tool must be version `0.N.x` (x can be anything)
    pub version: Option<u32>,

    /// Custom entry point symbol for the module. This should only be set if libmegaton is disabled.
    /// Using this disables the libmegaton runtime, which also means rust support will be disabled.
    pub custom_entry: Option<String>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Default for MegatonConfig {
    fn default() -> Self {
        Self {
            version: None,
            custom_entry: None,
            unused: Default::default(),
        }
    }
}

impl Validate for MegatonConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        cu::hint!("TODO: implement version check");
        self.unused.validate(ctx)
    }
}

impl MegatonConfig {
    /// Get the entry point symbol name
    pub fn entry_point(&self) -> &str {
        match &self.custom_entry {
            None => "__megaton_module_entry",
            Some(entry) => {
                if entry == "" {
                    "__megaton_module_entry"
                } else {
                    entry
                }
            }
        }
    }

    /// Checks if libmegaton enabled
    pub fn lib_enabled(&self) -> bool {
        self.custom_entry.clone().is_none_or(|val| val == "")
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
    #[serde(default = "default_target")]
    pub target: PathBuf,

    /// The compile_commands.json file to put/update compile commands
    /// for other tools like clangd
    #[serde(default = "default_comp_commands")]
    pub compdb: String,

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
                "'{}' is not a valid module name (must only contain alphanumeric characters, - or _",
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
}

fn default_target() -> PathBuf {
    PathBuf::from("target")
}

fn default_comp_commands() -> String {
    "compile_commands.json".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Set the profile to use when profile is unspecified
    ///
    /// If `Some("")`, a profile must be specified in command line or megaton will error
    pub default: Option<String>,

    /// Allow the base (`base`) profile to be used
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
            ("base", Some(p)) if p.is_empty() => {
                cu::error!("no profile specified!");
                cu::hint!("- please specify a profile with -p PROFILE");
                cu::bail!("failed to selected a profile");
            }
            ("base", Some(p)) => p,
            ("base", None) => BASE_PROFILE,
            (profile, _) => profile,
        };

        if !self.allow_base && profile == "base" {
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
    pub symbols: Vec<String>,
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
