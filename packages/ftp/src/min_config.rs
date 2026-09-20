use cu::pre::*;

/// Minimal Megaton.toml for ftp
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub module: ModuleConfig,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModuleConfig {
    pub name: String,
    pub title_id: u64,
    pub nso_name: Option<String>,
    pub target: Option<String>,
}

// TODO: in the config we may want an [ftp] section
// for uploading or downloading additional files
