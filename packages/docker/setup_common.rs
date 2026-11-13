use std::path::Path;

use cu::pre::*;


/// Copy a directory, respecting .dockerignore
///
/// This is for moving files in the repo when building the docker
/// container, since docker build does not allow accessing files
/// outside of the build context
pub fn copy_dir_for_docker(dir: &Path, to: &Path) -> cu::Result<()> {
    let mut walker = cu::fs::walker(dir);
    walker.git(false).add_ignore_filename(".dockerignore");
    for entry in walker.walk()? {
        let entry = entry?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let rel_path = entry_path.try_to_rel_from(dir);
        cu::ensure!(rel_path.is_relative(), "{}", rel_path.display())?;

        let target_path = to.join(rel_path);
        cu::fs::copy(entry_path, target_path)?;
    }

    Ok(())
}
