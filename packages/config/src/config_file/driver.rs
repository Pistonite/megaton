use std::path::PathBuf;

use cu::pre::*;
use semver::VersionReq;

use crate::config_file::{self, BASE_PROFILE, BuildConfig, CaptureUnused, MegatonConfig, ModuleConfig, Profile, ProjectTargetEnv, Resolve, RustConfig, Validate, ValidateCtx};
use crate::toolchain::ToolchainEnv;

/// The Megaton.toml config file
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The resolved project env
    #[serde(skip)]
    pub project: ProjectTargetEnv,

    /// `[module]` section
    pub module: ModuleConfig,

    /// `[megaton]` section
    #[serde(default)]
    pub megaton: MegatonConfig,

    /// `[rust]` section
    #[serde(default)]
    pub rust: RustConfig,

    /// `[build]` section
    #[serde(default)]
    build: Profile<BuildConfig>,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}
impl Config {
    pub fn new_from_project_dir(root_dir: Option<&str>, opts: ConfigLoadOpts<'_,'_>) -> cu::Result<Self> {
        let paths = ProjectPaths::resolve(root_dir)?;
        let content = cu::fs::read_string(&paths.config_path)?;
        let mut config = cu::check!(
            toml::parse::<Config>(&content),
            "failed to parse Megaton.toml"
        )?;
        if opts.resolve {
            config.resolve(paths.root, paths.config_path, opts.cli_profile, opts.toolchain)?;
        }
        if opts.validate {
            config.validate_root()?;
        }
        Ok(config)
    }
    pub fn new_no_megaton_toml(opts: ConfigLoadOpts<'_, '_>) -> cu::Result<Self> {
        let mut raw = Self {
            module: ModuleConfig::new_no_megaton_toml(),
            ..Default::default()
        };
        if let Ok(v) = VersionReq::parse(
            env!("CARGO_PKG_VERSION")
        ) {
            raw.megaton.version = Some(v);
        }
        if opts.resolve {
            raw.resolve(PathBuf::new(), PathBuf::new(), opts.cli_profile, opts.toolchain)?;
        }
        if opts.validate {
            raw.validate_root()?;
        }
        // reset the module name after validating since module name cannot be empty
        raw.module.name = String::new();
        Ok(raw)
    }
    #[cu::context("error resolving config")]
    fn resolve(&mut self, root: PathBuf, config_path: PathBuf, cli_profile: Option<&str>, toolchain: Option<&ToolchainEnv>) -> cu::Result<()> {
        let profile = self.select_profile(cli_profile)?;
        let target_root = cu::path!(&root / (self.module.target_dir) / "megaton");
        let project = ProjectTargetEnv::new(
            &self.module.name,
            profile.to_string(),
            root,
            config_path,
            target_root,
        );
        self.project = project;
        self.module.resolve(&self.project);
        self.rust.resolve(&self.project);
        cu::check!(self.build.resolve(&self.project, toolchain), "failed to resolve build config")?;
        Ok(())
    }
    /// Select profile based on command line and config
    ///
    /// Prints formatted error message on failure
    fn select_profile<'a, 'b>(&'a self, cli_profile: Option<&'b str>) -> cu::Result<&'a str>
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

        if profile != BASE_PROFILE && !config_file::is_profile_name_allowed(profile) {
            cu::bail!("selected profile {profile:?} is not a valid profile name");
        }

        Ok(profile)
    }
    pub fn build_config(&self) -> cu::Result<BuildConfig> {
        self.build.get_profile(&self.project.profile, "build")
    }
    /// Turn the config into a JSON blob after profile selection
    pub fn to_json(&self) -> cu::Result<json::Value> {
        let build = self.build_config()?;
        Ok(json!({
            "project": self.project,
            "module": self.module,
            "megaton": self.megaton,
            "rust": self.rust,
            "build": build,
        }))
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

pub struct ConfigLoadOpts<'a, 'b> {
    /// Resolve the config. This is required unless inspecting values before resolving
    pub resolve: bool,
    /// Validate the config.
    pub validate: bool,
    /// The CLI --profile selection
    pub cli_profile: Option<&'a str>,
    /// Optionally resolved toolchain environment
    pub toolchain: Option<&'b ToolchainEnv>
}

/// Project environment
struct ProjectPaths {
    /// Root of the project
    pub root: PathBuf,
    /// Path of the config file
    pub config_path: PathBuf,
}
impl ProjectPaths {
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
}
static MEGATON_TOML_FILENAMES: &[&str] = &["Megaton.toml", "megaton.toml"];
