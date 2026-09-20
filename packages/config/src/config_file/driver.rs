use std::path::PathBuf;

use cu::pre::*;
use semver::VersionReq;

use crate::{BASE_PROFILE, RustConfig};
use crate::config_file::{self, CaptureUnused, Validate, ValidateCtx, ModuleConfig, MegatonConfig};

/// Project environment
pub struct ProjectEnv {
    /// Root of the project
    pub root: PathBuf,
    /// Path of the config file
    pub config_path: PathBuf,
}
impl ProjectEnv {
    /// Resolve the project from --dir/-C flag
    pub fn resolve(root_dir: Option<&str>) -> cu::Result<Self> {
        if let Some(root_dir) = root_dir {
            let mut p = PathBuf::from(root_dir);
            for config_file in MEGATON_TOML_FILENAMES {
                p.push(config_file);
                if p.exists() {
                    return Ok(Self {
                        root: p.parent_abs()?,
                        config_path: p.normalize()?,
                    });
                }
                p.pop();
            }
            cu::bail!(
                "failed to resolve project root from '{root_dir}'; please ensure Megaton.toml exists"
            );
        }

        let cwd = PathBuf::from(".").normalize()?;
        for path in cwd.ancestors() {
            let mut p = path.to_path_buf();
            for config_file in MEGATON_TOML_FILENAMES {
                p.push(config_file);
                if p.exists() {
                    return Ok(Self {
                        root: p.parent_abs()?,
                        config_path: p.normalize()?,
                    });
                }
                p.pop();
            }
        }
        cu::bail!("failed to resolve project root; please ensure Megaton.toml exists");
    }
    #[cu::context("failed to load project config")]
    pub fn load_config(&self, resolve: bool, validate: bool) -> cu::Result<Config> {
        let content = cu::fs::read_string(&self.config_path)?;
        let mut config = cu::check!(
            toml::parse::<Config>(&content),
            "failed to parse Megaton.toml"
        )?;
        if resolve {
            config.root = self.root.clone();
            config.resolve();
        }
        if validate {
            config.validate_root()?;
        }
        Ok(config)
    }
}
static MEGATON_TOML_FILENAMES: &[&str] = &["Megaton.toml", "megaton.toml"];

/// The Megaton.toml config file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The resolved project root
    #[serde(skip)]
    pub root: PathBuf,
    pub module: ModuleConfig,
    #[serde(default)]
    pub megaton: MegatonConfig,
    #[serde(default)]
    pub rust: RustConfig,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}
impl Config {
    pub fn new_no_megaton_toml(resolve: bool, validate: bool) -> cu::Result<Self> {
        let mut raw = Self {
            module: ModuleConfig::new_no_megaton_toml(),
            ..Default::default()
        };
        if let Ok(v) = VersionReq::parse(
            env!("CARGO_PKG_VERSION")
        ) {
            raw.megaton.version = Some(v);
        }
        if resolve {
            raw.resolve();
        }
        if validate {
            raw.validate_root()?;
        }
        // reset the module name after validating since module name cannot be empty
        raw.module.name = String::new();
        Ok(raw)
    }
    fn resolve(&mut self) {
        self.module.resolve(&self.root);
        self.rust.resolve(&self.root);
    }
    /// Select profile based on command line and config
    ///
    /// Prints formatted error message on failure
    pub fn select_profile<'a, 'b>(&'a self, cli_profile: Option<&'b str>) -> cu::Result<&'a str>
    where
        'b: 'a, // lifetime: cli should live longer since that's parsed before config
    {
        let is_specified_in_cli = cli_profile.is_some();
        let profile =match cli_profile {
            None => {
            if self.module.default_profile.as_ref().is_some_and(|x| x.is_empty()) {
                cu::bail!(
                    "a profile must be selected from the command line (--profile PROFILE); required by module.default-profile = \"\""
                );
            }
                BASE_PROFILE
            }
            Some(p) => p
        };
        let profile = if profile == BASE_PROFILE {
            self
                .module
                .default_profile
                .as_deref()
                .unwrap_or(BASE_PROFILE)
        } else {
            profile
        };

        if !self.module.allow_base_profile && profile == BASE_PROFILE {
            if is_specified_in_cli {
                cu::bail!("the base profile (\"{BASE_PROFILE}\") is not allowed; please specify a profile from the command line (--profile PROFILE); required by module.allow-base-profile = false");
            } else {
                cu::bail!("the base profile (\"{BASE_PROFILE}\") is not allowed; required by module.allow-base-profile = false");
            }
        }

        Ok(profile)
    }
    /// Turn the config into a JSON blob after profile selection
    pub fn to_json(&self, profile: &str) -> cu::Result<json::Value> {
        Ok(json!({
            "root": self.root,
            "module": self.module,
            "megaton": self.megaton,
            "rust": self.rust,
        }))
    }
    /// Dump a value from the config
    pub fn dump(&self, key: &str, profile: &str) -> cu::Result<json::Value> {
        let mut value = self.to_json(profile)?;
        let mut rest = key;
        let mut current_path = String::new();
        loop {
            let next = cu::check!(
                config_file::parse_next_key(rest),
                "error parsing config key"
            )?;
            let key = match next {
                Some((key, next_rest)) => {
                    rest = next_rest;
                    key
                }
                None => return Ok(value),
            };
            match value {
                json::Value::Object(mut map) => match map.remove(key) {
                    None => {
                        let keys = format!("{:?}", map.keys().collect::<Vec<_>>());
                        cu::bail!(
                            "invalid key '{key}' at '{current_path}', valid keys are: {keys}"
                        );
                    }
                    Some(v) => value = v,
                },
                json::Value::Array(mut values) => {
                    let len = values.len();
                    let num = cu::parse::<usize>(key.trim());
                    let num = cu::check!(
                        num,
                        "invalid key '{key}' at '{current_path}', valid keys are: [0 - {}] for array of length {len}",
                        len - 1
                    )?;
                    if num >= len {
                        cu::bail!("index {num} out of bound: length at '{current_path}' is {len}");
                    }
                    value = values.swap_remove(num);
                }
                x => {
                    let stringified = match json::stringify(&x) {
                        Ok(x) => x,
                        Err(_) => format!("{x:?}"),
                    };
                    cu::bail!(
                        "invalid key '{key}' at '{current_path}', is a scalar value: {stringified}"
                    );
                }
            }
            if !current_path.is_empty() {
                current_path.push('.');
            }
            current_path.push_str(key);
        }
    }
}
impl Validate for Config {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.module.validate_property(ctx, "module")?;
        self.megaton.validate_property(ctx, "megaton")?;
        self.rust.validate_property(ctx, "rust")?;
        self.unused.validate(ctx)
    }
}
