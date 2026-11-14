// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

use std::path::Path;

use cu::pre::*;

use megaton_packaging as packaging;

pub fn setup_isolated_crates(image_dir: &str, package_dir_names: &[&str]) -> cu::Result<()> {
    let md = cu::env_var("CARGO_MANIFEST_DIR").unwrap_or_default();
    if md.is_empty() {
        cu::bail!("failed to get CARGO_MANIFEST_DIR, please run the packaging script with cargo");
    }
    let image_dir_path = cu::path!(&(&md) / image_dir);
    if !image_dir_path.exists() {
        cu::bail!(
            "{image_dir} is not a valid image dir! (packages/docker/{image_dir} does not exist)"
        );
    }
    let crates_dir = cu::path!(image_dir_path / "crates");
    cu::fs::make_dir_empty(&crates_dir)?;

    let packages_list = package_dir_names
        .iter()
        .map(|x| format!("packages/{x}"))
        .collect::<Vec<_>>();
    let packages_list_ref = packages_list.iter().map(|x| x.as_str()).collect::<Vec<_>>();

    let packages_dir = Path::new(&md).parent_abs()?;
    let repo_dir = packages_dir.parent_abs()?;

    let workspace_manifest_path = repo_dir.join("Cargo.toml");
    let subset_workspace_manifest =
        packaging::create_subset_workspace_manifest(&workspace_manifest_path, &packages_list_ref)?;
    let subset_workspace_manifest_path = crates_dir.join("Cargo.toml");
    cu::fs::write(&subset_workspace_manifest_path, &subset_workspace_manifest)?;
    cu::fs::copy(repo_dir.join("Cargo.lock"), crates_dir.join("Cargo.lock"))?;

    for package in package_dir_names {
        copy_dir_for_docker(
            &packages_dir.join(package),
            &crates_dir.join(format!("packages/{package}")),
        )?;
    }

    Ok(())
}

/// Copy a directory, respecting .dockerignore
///
/// This is for moving files in the repo when building the docker
/// container, since docker build does not allow accessing files
/// outside of the build context
fn copy_dir_for_docker(dir: &Path, to: &Path) -> cu::Result<()> {
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
