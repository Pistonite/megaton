//! Config structures
use buildcommon::{prelude::*, Unused};

use std::collections::BTreeMap;
use std::path::Path;

use buildcommon::flags::FlagConfig;
use serde::{Deserialize, Serialize};

use crate::error::Error;

fn default_true() -> bool {
    true
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

    /// The `[build]` section
    ///
    /// Specify options for building C/C++/assembly project code
    /// and linking the module
    pub build: ProfileContainer<Build>,

    /// The `[check]` section (for checking unresolved dynamic symbols)
    pub check: Option<ProfileContainer<Check>>,

    #[serde(flatten, default)]
    unused: Unused,
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
        let config: Self = toml::from_str(&config).inspect_err(|e| {
            for line in e.to_string().lines() {
                errorln!("Error", "{}", line);
            }
        })?;

        config.unused.check();
        config.module.unused.check_prefixed("module");
        config.profile.unused.check_prefixed("profile");

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

    #[serde(flatten, default)]
    unused: Unused,
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
    unused: Unused,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            default: None,
            allow_base: true,
            unused: Unused::default(),
        }
    }
}

impl ProfileConfig {
    /// Select profile based on command line and config
    ///
    /// Prints formatted error message on failure
    pub fn resolve<'a, 'b>(&'a self, cli_profile: &'b str) -> Result<&'a str, Error>
    where
        'b: 'a, // lifetime: cli should live longer since that's parsed before config
    {
        let profile = match (cli_profile, &self.default) {
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

        if !self.allow_base && profile == "none" {
            errorln!("Error", "Base profile is disallowed");
            hintln!(
                "Consider",
                "Set `profile.default` in config or specify a profile with `--profile`"
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
//

/// Config in the `[build]` section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Build {
    /// Whether to use libmegaton (default is true)
    ///
    /// If false, the module won't link with libmegaton
    /// and you won't be able to use megaton's runtime features. This is useful
    /// if you want to bring your own runtime. Note that this is required for
    /// Rust support
    ///
    /// If libmegaton is not used, you also must set [`entry`](Self::entry) to the
    /// name of the entrypoint function
    #[serde(default = "default_true")]
    pub libmegaton: bool,

    /// Entry point symbol for the module. Only used if `libmegaton` is false
    pub entry: Option<String>,

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

    #[serde(default)]
    pub flags: FlagConfig,

    #[serde(flatten, default)]
    unused: Unused,
}

impl Build {
    /// Validate config values
    pub fn check(&self) -> Result<(), Error> {
        verboseln!("build: {:?}", self);
        if self.libmegaton {
            if self.entry.is_some() {
                errorln!("Error", "Entry point specified with libmegaton enabled");
                hintln!("Consider", "Set `build.libmegaton` to false if you want to use your own entry point with no runtime");
                return Err(report!(Error::BadRuntime))
                    .attach_printable("entry point should not be specified when using libmegaton");
            }
        } else if self.entry.is_none() {
            errorln!("Error", "Entry point not specified");
            hintln!(
                "Consider",
                "Set `build.libmegaton` to true to use libmegaton runtime"
            );
            hintln!("Consider", "Or specify an entry point with `build.entry`");
            return Err(report!(Error::BadRuntime))
                .attach_printable("entry point must be specified when not using libmegaton");
        }
        self.unused.check_prefixed("build");
        self.flags.unused.check_prefixed("build.flags");
        Ok(())
    }

    pub fn entry_point(&self) -> &str {
        match &self.entry {
            Some(entry) => entry,
            None => "__megaton_module_entry",
        }
    }
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

    #[serde(flatten, default)]
    unused: Unused,
}

impl Check {
    pub fn check(&self) {
        self.unused.check_prefixed("check");
    }
}

impl Profilable for Check {
    fn extend(&mut self, other: &Self) {
        self.ignore.extend(other.ignore.iter().cloned());
        self.symbols.extend(other.symbols.iter().cloned());
        self.disallowed_instructions
            .extend(other.disallowed_instructions.iter().cloned());
    }
}
