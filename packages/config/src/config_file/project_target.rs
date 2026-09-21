use std::path::PathBuf;

use cu::pre::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectTargetEnv {
    /// Selected profile
    pub profile: String,

    /// Root of the project
    pub root: PathBuf,
    /// Path of the config file
    pub config_path: PathBuf,
    /// Megaton's target root (target/megaton)
    pub target_root: PathBuf,
    /// Root for megaton to emit library files
    pub lib_root: PathBuf,
    /// Root for megaton to emit output files for this project/profile
    pub output_root: PathBuf,
}

impl ProjectTargetEnv {
    pub fn new(
        name: &str,
        profile: String,
        root: PathBuf,
        config_path: PathBuf,
        target_root: PathBuf,
    ) -> Self {
        let lib_root = cu::path!(&target_root / "lib");
        let output_root = cu::path!(&target_root / profile / name);
        Self {
            profile,
            root,
            config_path,
            target_root,
            lib_root,
            output_root,
        }
    }
}
