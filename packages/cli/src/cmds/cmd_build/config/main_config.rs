// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

//! Config structures
use std::path::Path;

use super::{BASE_PROFILE, Build, CaptureUnused, ExtendProfile, Profile, Validate, ValidateCtx};
use cu::pre::*;

/// Load a Megaton.toml config file
pub fn load_config(path: impl AsRef<Path>) -> cu::Result<Config> {
    let content = cu::fs::read_string(path)?;
    let config = toml::parse::<Config>(&content).context("failed to parse Megaton config")?;
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
        cu::hint!("TODO: add cargo config");
        self.build.validate_property(ctx, "build")?;
        if let Some(check) = &self.check {
            check.validate_property(ctx, "check")?;
        }

        self.unused.validate(ctx)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CargoConfig {
    // TODO: implement
}

// impl Default for CargoConfig {
//     fn default() -> Self {
//         Self {
//             // TODO: Implement
//         }
//     }
// }

/// The `[megaton]` section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MegatonConfig {
    /// If specified, megaton will run version check
    ///
    /// `N` means the build tool must be version `0.N.x` (x can be anything)
    pub version: Option<u32>,

    /// Whether to use libmegaton (default is true)
    ///
    /// If false, the module won't link with libmegaton
    /// and you won't be able to use megaton's runtime features. This is useful
    /// if you want to bring your own runtime. Note that this is required for
    /// Rust support
    ///
    /// If libmegaton is not used, you also must set `entry` to the
    /// name of the entrypoint function
    #[serde(default = "default_true")]
    pub library: bool,

    /// Entry point symbol for the module. Only allowed if `megaton` is false
    pub entry: Option<String>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Default for MegatonConfig {
    fn default() -> Self {
        Self {
            version: None,
            library: true,
            entry: None,
            unused: Default::default(),
        }
    }
}

impl Validate for MegatonConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        cu::hint!("TODO: implement version check");
        if self.library {
            if self.entry.is_some() {
                cu::error!("megaton.entry can only be used when megaton.library = false!");
                cu::hint!(
                    "- consider setting megaton.library to false if you want to use your own library instead of megaton."
                );
                ctx.bail()?;
            }
        } else if self.entry.is_none() {
            cu::error!("megaton.entry must be specified when megaton.library = false!");
            cu::hint!(
                "- consider specifying build.entry to the symbol of your module's entry point"
            );
            cu::hint!("- alternatively, set megaton.library to true to use megaton library");
            ctx.bail()?;
        }
        self.unused.validate(ctx)
    }
}

impl MegatonConfig {
    /// Get the entry point symbol name
    pub fn entry_point(&self) -> &str {
        match &self.entry {
            Some(entry) => entry,
            None => "__megaton_module_entry",
        }
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
    pub target: Option<String>,

    /// The compile_commands.json file to put/update compile commands
    /// for other tools like clangd
    pub compdb: Option<String>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Validate for Module {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        if self.name.is_empty() {
            cu::bailfyi!("module.name must be non-empty");
        }
        if self.name == "lib" {
            cu::bailfyi!("'lib' is reserved and cannot be the module name");
        }
        if self
            .name
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
        {
            cu::bailfyi!(
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Set the profile to use when profile is "none"
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
                cu::bailfyi!("failed to selected a profile");
            }
            ("none", Some(p)) => p,
            ("none", None) => BASE_PROFILE,
            (profile, _) => profile,
        };

        if !self.allow_base && profile == "none" {
            cu::error!("base profile is not allowed!");
            cu::hint!("- please specify a profile with -p PROFILE");
            cu::bailfyi!("failed to selected a profile");
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
