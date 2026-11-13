use std::path::Path;

use cu::pre::*;

use megaton_packaging as packaging;
use megaton_docker_setup;

#[cu::cli]
fn main(_: cu::cli::Flags) -> cu::Result<()> {
    let md = cu::env_var("CARGO_MANIFEST_DIR").unwrap_or_default();
    if md.is_empty() {
        cu::bail!("failed to get CARGO_MANIFEST_DIR, please run the packaging script with cargo");
    }
    let crates_dir = cu::path!(&(&md) / "toolchain-image" / "crates");
    cu::fs::make_dir_empty(&crates_dir)?;

    let packages_dir = Path::new(&md).parent_abs()?;
    let repo_dir = packages_dir.parent_abs()?;

    let workspace_manifest_path = repo_dir.join("Cargo.toml");
    let subset_workspace_manifest = packaging::create_subset_workspace_manifest(
        &workspace_manifest_path,
        &["packages/toolchain", "packages/config"],
    )?;
    let subset_workspace_manifest_path = crates_dir.join("Cargo.toml");
    cu::fs::write(&subset_workspace_manifest_path, &subset_workspace_manifest)?;

    megaton_docker_setup::copy_dir_for_docker(
        &packages_dir.join("toolchain"),
        &crates_dir.join("toolchain")
    )?;

    megaton_docker_setup::copy_dir_for_docker(
        &packages_dir.join("config"),
        &crates_dir.join("config")
    )?;


    Ok(())
}
