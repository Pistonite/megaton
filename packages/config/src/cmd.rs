use cu::pre::*;

use crate::{BASE_PROFILE, Config, ProjectEnv};

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
    #[clap(short='H', long)]
    hex: bool,

    #[clap(short, long, default_value = "raw")]
    format: ConfigDumpFormat,

    #[clap(flatten)]
    #[as_ref]
    common: cu::cli::Flags,
}
impl Cmd {
    pub fn run(self, dir: Option<&str>) -> cu::Result<()> {
        cu::lv::disable_print_time();
        let value = if self.env {
            todo!();
        } else {
            let config = if self.no_config {
                Config::new_no_megaton_toml(!self.no_resolve, !self.no_validate)?
            } else {
                let project = ProjectEnv::resolve(dir)?;
                project.load_config(!self.no_resolve, !self.no_validate)?
            };

            let profile = config.select_profile(self.profile.as_deref())?;


            let key = self.key.as_deref().unwrap_or_default();
            let value = cu::check!(config.dump(key, profile), "failed to get config value by key: '{key}'")?;
            value
        };

        match self.format {
            ConfigDumpFormat::Json => {
                let x = json::stringify(&value)?;
                println!("{x}");
            }
            ConfigDumpFormat::JsonPretty => {
                let x = json::stringify_pretty(&value)?;
                println!("{x}");
            }
            ConfigDumpFormat::Raw => {
                let x = json_obj_to_raw(&value, '\n', self.hex)?;
                println!("{x}");
            }
            ConfigDumpFormat::OneLine => {
                let x = json_obj_to_raw(&value, ' ', self.hex)?;
                println!("{x}");
            }
        }
        Ok(())
    }
}

impl AsProfileFlag for Cmd {
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>> {
        Some(&mut self.profile)
    }
}

#[derive(clap::ValueEnum, Default, Clone)]
pub enum ConfigDumpFormat {
    /// One-line JSON
    Json,
    /// Pretty JSON
    JsonPretty,
    /// Raw: arrays are dumped as one value per line; objects are dumped as one `key=value` per
    /// line; over-complex objects cannot be dumped
    #[default]
    Raw,
    /// Like Raw, but array and objects are space-separated instead of one per line.
    OneLine
}

fn json_obj_to_raw(value: &json::Value, join: char, hex: bool) -> cu::Result<String> {
    let mut buf = String::new();
    match value {
        json::Value::Array(values) => {
            for v in values {
                if !buf.is_empty() {
                    buf.push(join);
                }
                json_to_raw(&mut buf, v, hex)?;
            }
        }
        json::Value::Object(map) => {
            for (k,v) in map {
                if !buf.is_empty() {
                    buf.push(join);
                }
                buf.push_str(k);
                buf.push('=');
                json_to_raw(&mut buf, v, hex)?;
            }
        }
        other => {
            json_to_raw(&mut buf, other, hex)?;
        }
    }
    Ok(buf)
}

fn json_to_raw(out: &mut String, value: &json::Value, hex: bool) -> cu::Result<()> {
    match value {
        json::Value::Object(_) | json::Value::Array(_) => {
            cu::bail!("object is too complex; please use --format=json or --format=json-pretty");
        }
        json::Value::Null => out.push_str("null"),
        json::Value::Bool(x) => {
            if *x {
                out .push_str( "true");
            } else {
                out .push_str( "false");
            }
        }
        json::Value::Number(n) => {
            use std::fmt::Write;
            if hex && let Some(x) = n.as_u64() {
                let _ = write!(out, "0x{x:x}");
            } else {
                let _ = write!(out, "{n}");
            }
        }
        json::Value::String(x) => {
            out.push_str(x)
        }
    }
    Ok(())
}

pub trait AsProfileFlag {
    /// Get a mutable reference to the --profile flag, if the command supports
    /// it
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>>;
}
