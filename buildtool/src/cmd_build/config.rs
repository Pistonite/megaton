//! Config structures
use buildcommon::prelude::*;

use std::collections::BTreeMap;
use std::path::Path;

use buildcommon::flags::FlagConfig;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Config data read from Megaton.toml
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The `[module]` section
    pub module: Module,

    /// The `[build]` section
    pub build: ProfileContainer<Build>,

    /// The `[check]` section (for checking unresolved dynamic symbols)
    pub check: Option<ProfileContainer<Check>>,
}

error_context!(pub LoadConfig, |r| -> Error {
    errorln!("Failed", "Loading Megaton.toml");
    r.change_context(Error::Config)
});
impl Config {
    /// Load a config from a file
    ///
    /// Prints formatted error message when failed
    pub fn from_path(path: impl AsRef<Path>) -> ResultIn<Self, LoadConfig> {
        let config = system::read_file(path)?;
        // print pretty toml error
        let config = toml::from_str(&config).map_err(|e| {
            for line in e.to_string().lines() {
                errorln!("Error", "{}", line);
            }
            e
        })?;

        Ok(config)
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
    /// Set the profile to use when profile is "none"
    ///
    /// If `Some("")`, a profile must be specified in command line or megaton will error
    pub default_profile: Option<String>,

    /// Disable the base (`none`) profile from being used
    #[serde(default)]
    pub disallow_base_profile: bool,
}

impl Module {
    /// Get the title ID as a lower-case hex string (without the `0x` prefix)
    pub fn title_id_hex(&self) -> String {
        format!("{:016x}", self.title_id)
    }

    /// Select profile based on command line and config
    ///
    /// Prints formatted error message on failure
    pub fn select_profile<'a, 'b>(&'a self, cli_profile: &'b str) -> Result<&'a str, Error>
    where
        'b: 'a, // lifetime: cli should live longer since that's parsed before config
    {
        let profile = match (cli_profile, &self.default_profile) {
            ("none", Some(p)) if p.is_empty() => {
                // default-profile = "" means to disallow no profile
                errorln!("Error", "No profile specified");
                hintln!("Consider", "Specify a profile with `--profile`");
                return Err(report!(Error::NoProfile))
                    .attach_printable("Please specify a profile with `--profile`");
            }
            ("none", Some(p)) => p,
            ("none", None) => "none",
            (profile, _) => profile,
        };

        if self.disallow_base_profile && profile == "none" {
            errorln!("Error", "Base profile is disallowed");
            hintln!(
                "Consider",
                "Set `module.default-profile` in config or specify a profile with `--profile`"
            );

            return Err(report!(Error::NoProfile));
        }

        Ok(profile)
    }
}

// /// Config in the `[rust]` section
// #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "kebab-case")]
// pub struct Rust {
//     /// If the module should be built without linking to the std crate.
//     ///
//     /// If true, the target will be aarch64-nintendo-switch-freestanding. Otherwise it
//     /// will be aarch64-unknown-hermit and the binary will include the hermit kernel.
//     pub no_std: Option<bool>,
//
//     /// Additional build flags to pass to cargo
//     #[serde(default)]
//     pub build_flags: Vec<String>,
// }

// impl Profilable for Rust {
//     fn extend(&mut self, other: &Self) {
//         if let Some(no_std) = other.no_std {
//             self.no_std = Some(no_std);
//         }
//     }
// }

/// Config in the `[build]` section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Build {
    /// Entry point symbol for the module
    ///
    /// This value is not optional, but marked as optional so
    /// we give a more descriptive error (rather than a TOML parse error)
    pub entry: Option<String>,

    /// C/C++ Source directories, relative to Megaton.toml
    #[serde(default)]
    pub sources: Vec<String>,

    /// C/C++ Include directories, relative to Megaton.toml
    #[serde(default)]
    pub includes: Vec<String>,

    /// Library paths
    #[serde(default)]
    pub libpaths: Vec<String>,

    /// Libraries to link with
    #[serde(default)]
    pub libraries: Vec<String>,

    /// Linker scripts
    #[serde(default)]
    pub ldscripts: Vec<String>,

    pub flags: FlagConfig,
}

impl Profilable for Build {
    fn extend(&mut self, other: &Self) {
        if let Some(entry) = other.entry.clone() {
            self.entry = Some(entry);
        }
        self.sources.extend(other.sources.iter().cloned());
        self.includes.extend(other.includes.iter().cloned());
        self.libpaths.extend(other.libpaths.iter().cloned());
        self.libraries.extend(other.libraries.iter().cloned());
        self.ldscripts.extend(other.ldscripts.iter().cloned());
        self.flags.extend(&other.flags);
    }
}

impl Profilable for FlagConfig {
    fn extend(&mut self, other: &Self) {
        extend_flags(&mut self.common, &other.common);
        extend_flags(&mut self.c, &other.c);
        extend_flags(&mut self.cxx, &other.cxx);
        extend_flags(&mut self.as_, &other.as_);
        extend_flags(&mut self.ld, &other.ld);
    }
}

fn extend_flags(dst: &mut Option<Vec<String>>, src: &Option<Vec<String>>) {
    match (dst.as_mut(), src) {
        (_, None) => {}
        (None, Some(flags)) => {
            // dst none = ["<default>"]
            let mut new_flags = flags.clone();
            if !new_flags.iter().any(|x| x == "<default>") {
                new_flags.push("<default>".to_string());
            }
            *dst = Some(new_flags);
        }
        (Some(dst_flags), Some(src_flags)) => {
            for flag in src_flags {
                if !dst_flags.contains(flag) {
                    dst_flags.push(flag.clone());
                }
            }
        }
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
}

impl Profilable for Check {
    fn extend(&mut self, other: &Self) {
        self.ignore.extend(other.ignore.iter().cloned());
        self.symbols.extend(other.symbols.iter().cloned());
        self.disallowed_instructions
            .extend(other.disallowed_instructions.iter().cloned());
    }
}

/// Generic config section that can be extended with profiles
///
/// For example, the `[make]` section can have profiles with `[make.profiles.<name>]`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProfileContainer<T>
where
    T: Profilable + Clone,
{
    /// The base profile
    #[serde(flatten)]
    base: T,
    /// The extended profiles
    #[serde(default)]
    profiles: BTreeMap<String, T>,
}

impl<T> ProfileContainer<T>
where
    T: Profilable + Clone,
{
    /// Get a profile by name
    ///
    /// If the name is "none", or there is no profile with that name,
    /// the base profile will be returned. Otherwise, returns the base profile
    /// extended with the profile with the given name.
    pub fn get_profile(&self, name: &str) -> T {
        let mut base = self.base.clone();
        if name != "none" {
            if let Some(profile) = self.profiles.get(name) {
                base.extend(profile);
            }
        }
        base
    }
}

/// A trait for extending a config section with a profile
pub trait Profilable {
    /// Extend this config section with another
    fn extend(&mut self, other: &Self);
}
