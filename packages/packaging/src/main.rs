use std::path::Path;

use cu::pre::*;

use megaton_packaging as packaging;
use packaging::LibraryPackager;

#[cu::cli]
fn main(_: cu::cli::Flags) -> cu::Result<()> {
    let md = cu::env_var("CARGO_MANIFEST_DIR").unwrap_or_default();
    if md.is_empty() {
        cu::bail!("failed to get CARGO_MANIFEST_DIR, please run the packaging script with cargo");
    }
    let dist_dir = Path::new(&md).join("dist");
    cu::fs::make_dir_empty(&dist_dir)?;

    let packages_dir = Path::new(&md).parent_abs()?;

    if !cu::path!(&packages_dir / "mega-cpp" / "lib" / "libmegacpp.a").exists() {
        cu::bail!(
            "mega-cpp is not built, please run `task make` instead of the packaging script directly"
        );
    }

    cu::info!("located packages directory: {}", packages_dir.display());

    let repo_dir = packages_dir.parent_abs()?;
    let workspace_manifest_path = repo_dir.join("Cargo.toml");

    let subset_workspace_manifest = packaging::create_subset_workspace_manifest(
        &workspace_manifest_path,
        &["packages/mega-rs", "packages/mega-rspmacro"],
    )?;
    let subset_workspace_manifest_path = dist_dir.join("Cargo.toml");

    cu::fs::write(&subset_workspace_manifest_path, &subset_workspace_manifest)?;
    cu::info!("created manifest for workspace subset packages");

    let mut packer = LibraryPackager::new();
    packer.add_file(repo_dir.join("Cargo.lock"), "Cargo.lock")?;
    packer.add_file(subset_workspace_manifest_path, "Cargo.toml")?;

    packer.add_dir(packages_dir.join("mega-rs"), "packages/mega-rs", false)?;
    packer.add_dir(
        packages_dir.join("mega-rspmacro"),
        "packages/mega-rspmacro",
        false,
    )?;
    packer.add_dir(packages_dir.join("mega-cpp"), "mega-cpp", false)?;

    let info = packer.finish(
        dist_dir.join("megatonlib.tar.gz"),
        dist_dir.join("megatonlib.tar.gz.sha256"),
    )?;

    cu::info!("successfully packed megatonlib.tar.gz");
    cu::info!("size: {} bytes, sha256: {}", info.size, info.sha256);

    Ok(())
}
