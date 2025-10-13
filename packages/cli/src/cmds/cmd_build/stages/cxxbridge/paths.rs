use std::path::{Path, PathBuf};
use super::discover::LibInfo;

pub fn include_root(project_root: &Path) -> PathBuf {
    project_root.join("build").join("include")
}

pub fn map_outputs(project_root: &Path, lib: &LibInfo, rs: &Path) -> cu::Result<(PathBuf, PathBuf, PathBuf)> {
    let rel = match pathdiff::diff_paths(rs, &lib.src_dir) {
        Some(r) => r,
        None => cu::bail!("failed to compute path relative to src/"),
    };    
    let header = include_root(project_root).join(&lib.crate_name).join(&rel).with_extension("rs.h");
    let cc: PathBuf = project_root.join("build").join("cxx").join(&lib.crate_name).join(&rel).with_extension("rs.cc");
    let obj: PathBuf = project_root.join("build").join("obj").join(&lib.crate_name).join(&rel).with_extension("rs.o");
    Ok((header, cc, obj))
}
