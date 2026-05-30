// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::path::{Path, PathBuf};

use cu::pre::*;

fn main() -> cu::Result<()> {
    prep_toolchain_build_crate()?;
    prep_megaton_cmd_crate()?;
    Ok(())
}
fn prep_toolchain_build_crate() -> cu::Result<()> {
    let toolchain_manifest_path = {
        let mut p = packages_path()?;
        p.extend(["toolchain-build", "Cargo.toml"]);
        p
    };

    let isolated_manifest = cu::check!(
        megaton_toolchain_build::create_isolated_cargo_manifest(&toolchain_manifest_path, None),
        "failed to create isolated manifest for megaton-toolchain-build"
    )?;
    cu::fs::write("temp/toolchain-build/Cargo.toml", &isolated_manifest)?;

    Ok(())
}
fn prep_megaton_cmd_crate() -> cu::Result<()> {
    let commit_hash = megaton_cli_build::get_commit()?;
    let packages_path = packages_path()?;
    let info = cu::check!(
        megaton_cli_build::pack_library(
            &packages_path,
            Path::new("temp/megaton-cmd/libmegaton.tar.gz")
        ),
        "failed to pack library"
    )?;
    let library_hash = info.sha256;

    let cli_manifest_path = {
        let mut p = packages_path;
        p.extend(["cli", "Cargo.toml"]);
        p
    };

    let isolated_manifest = cu::check!(
        megaton_toolchain_build::create_isolated_cargo_manifest_with_deps_removed(
            &cli_manifest_path,
            None,
            ["megaton-cli-build",]
        ),
        "failed to create isolated manifest for megaton-cmd"
    )?;

    cu::fs::write("temp/megaton-cmd/Cargo.toml", &isolated_manifest)?;
    cu::fs::write(
        "temp/megaton-cmd/build.rs",
        format!(
            r##"
fn main() {{
    println!("cargo::rustc-env=MEGATON_COMMIT={commit_hash}");
    println!("cargo::rustc-env=MEGATON_LIB_SHA256={library_hash}");
}}
"##
        ),
    )?;
    Ok(())
}
/// get the <repo>/packages directory
fn packages_path() -> cu::Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent_abs()
}
