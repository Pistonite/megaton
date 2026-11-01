use std::path::PathBuf;

use cu::pre::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectTargetEnv {
    /// Selected profile
    pub profile: String,
    pub module_name: String,
    pub title_id: u64,
    /// Upload nso name
    pub nso_name: Option<String>,

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
    /// The output elf path
    pub out_elf: PathBuf,
    /// The output nso path
    pub out_nso: PathBuf,
    /// The output npdm path
    pub out_npdm: PathBuf,
    /// Output generated source paths
    pub out_src: PathBuf,
    /// Output generated include paths
    pub out_inc: PathBuf,
    /// Output object paths
    pub out_objects: PathBuf,
}

impl ProjectTargetEnv {
    pub fn new(
        name: &str,
        title_id: u64,
        nso_name: Option<String>,
        profile: String,
        root: PathBuf,
        config_path: PathBuf,
        target_root: PathBuf,
    ) -> Self {
        let lib_root = cu::path!(&target_root / "lib");
        let output_root = cu::path!(&target_root / profile / name);
        let out_elf = cu::path!(&output_root / format!("{name}.elf"));
        let out_nso = cu::path!(&output_root / format!("{name}.nso"));
        let out_npdm = cu::path!(&output_root / "main.npdm");
        let out_src = cu::path!(&output_root / "src");
        let out_inc = cu::path!(&output_root / "inc");
        let out_objects = cu::path!(&output_root / "o");
        Self {
            profile,
            module_name: name.to_string(),
            title_id,
            nso_name,
            root,
            config_path,
            target_root,
            lib_root,
            output_root,
            out_elf,
            out_nso,
            out_npdm,
            out_src,
            out_inc,
            out_objects,
        }
    }
}
