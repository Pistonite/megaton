use cu::pre::*;

use crate::config_file::{BASE_PROFILE, Config, ConfigLoadOpts};
use crate::dump::{self, DumpFormat};
use crate::toolchain::ToolchainEnv;

#[derive(clap::Parser, AsRef)]
pub struct Cmd {
    /// The key path to dump in the config object. [default: dumps whole config]
    #[clap(short, long)]
    key: Option<String>,

    /// Dump the environment config instead of the project config
    #[clap(long)]
    env: bool,

    /// The profile to dump the config for
    #[clap(short = 'p', long, conflicts_with = "env", default_value = BASE_PROFILE)]
    profile: Option<String>,

    /// Do not try to resolve Megaton.toml config.
    ///
    /// This can be used to dump the default values from Megaton.
    /// Some keys will have useless placeholder values.
    #[clap(long, conflicts_with = "env")]
    no_config: bool,

    /// Do not run the resolve phase when loading the config.
    ///
    /// The resolve phase resolves paths and default tokens in the config.
    #[clap(long, conflicts_with = "env")]
    no_resolve: bool,

    /// Skip validating the config
    #[clap(long)]
    no_validate: bool,

    /// Format non-neg integers as hex in raw or one-line format
    #[clap(short = 'H', long)]
    hex: bool,

    #[clap(short, long, default_value = "raw")]
    format: DumpFormat,

    #[clap(flatten)]
    #[as_ref]
    common: cu::cli::Flags,
}
impl Cmd {
    pub fn run(self, dir: Option<&str>) -> cu::Result<()> {
        cu::lv::disable_print_time();
        cu::cli::print_to(cu::cli::Target::Stderr);
        let key = self.key.as_deref().unwrap_or_default();
        let value = if self.env {
            let toolchain = ToolchainEnv::resolve()?;
            let value = cu::check!(
                toolchain.to_json(),
                "failed to convert toolchain environment to JSON"
            )?;
            value
        } else {
            let toolchain = if self.no_resolve {
                None
            } else {
                let toolchain = ToolchainEnv::resolve()?;
                Some(toolchain)
            };
            let opts = ConfigLoadOpts {
                resolve: !self.no_resolve,
                validate: !self.no_validate,
                cli_profile: self.profile.as_deref(),
                toolchain: toolchain.as_ref(),
            };
            let config = if self.no_config {
                Config::new_no_megaton_toml(opts)?
            } else {
                Config::new_from_project_dir(dir, opts)?
            };

            let value = cu::check!(config.to_json(), "failed to convert config value to JSON")?;
            value
        };
        let value = cu::check!(
            dump::dump(key, value),
            "failed to dump value by key: '{key}'"
        )?;

        let dumped = self.format.stringify(&value, self.hex)?;
        println!("{dumped}");

        Ok(())
    }
}

impl AsProfileFlag for Cmd {
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>> {
        Some(&mut self.profile)
    }
}

pub trait AsProfileFlag {
    /// Get a mutable reference to the --profile flag, if the command supports
    /// it
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>>;
}
