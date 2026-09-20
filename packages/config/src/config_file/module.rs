use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::config_file::{CaptureUnused, Validate, ValidateCtx};

/// `[module]` config section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModuleConfig {
    /// The name of the module, used as the target name of the final binary.
    pub name: String,
    /// Any string for tracking the version of the module. Megaton ignores this.
    #[serde(default)]
    pub version: String,
    /// The title ID as a 64-bit integer, used for generating the npdm file.
    pub title_id: u64,

    /// Name of the module to upload with ftp (does not upload if not specified)
    pub nso_name: Option<String>,

    /// Set the profile to use when profile is unspecified
    ///
    /// If `Some("")`, a profile must be specified in command line or megaton will error
    pub default_profile: Option<String>,

    /// Allow the base (`none`) profile to be used
    #[serde(default = "ModuleConfig::default_allow_base")]
    pub allow_base_profile: bool,

    /// The target directory to put build files
    #[serde(default = "ModuleConfig::default_target")]
    pub target_dir: PathBuf,

    /// The compile_commands.json file to put/update compile commands
    /// for other tools like clangd
    #[serde(default = "ModuleConfig::default_compile_commands")]
    pub compile_commands: PathBuf,

    /// Arbitrary metadata that other tools can use 
    pub metadata: Option<json::Value>,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}

impl ModuleConfig {
    pub fn new_no_megaton_toml() -> Self {
        Self {
            name: "no-megaton-toml-placeholder".to_string(),
            target_dir: Self::default_target(),
            compile_commands: Self::default_compile_commands(),
            allow_base_profile: Self::default_allow_base(),
            ..Default::default()
        }
    }
    pub fn resolve(&mut self, root: &Path) {
        self.target_dir = root.join(&self.target_dir);
        self.compile_commands = root.join(&self.compile_commands);
    }
    /// Get the title ID as a lower-case hex string (without the `0x` prefix)
    pub fn title_id_hex(&self) -> String {
        format!("{:016x}", self.title_id)
    }
    fn default_target() -> PathBuf {
        "target".into()
    }
    fn default_compile_commands() -> PathBuf {
        "compile_commands.json".into()
    }
    fn default_allow_base() -> bool {
        true
    }
}

impl Validate for ModuleConfig {
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

