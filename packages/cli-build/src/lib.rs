// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::fs::File;
use std::path::Path;

use cu::pre::*;
use flate2::{Compression, write::GzEncoder};
use ignore::{Walk as IgnoreWalk, WalkBuilder as IgnoreWalkBuilder};
use sha2::{Digest, Sha256};
use tar::{Builder as TarBuilder, HeaderMode};

pub struct PackLibraryInfo {
    pub sha256: String,
}

#[cu::context("failed to get current commit")]
pub fn get_commit() -> cu::Result<String> {
    let (child, commit_hash) = cu::which("git")?
        .command()
        .args(["rev-parse", "HEAD"])
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let commit_hash = commit_hash.join()??;
    Ok(commit_hash.trim().to_string())
}

pub fn pack_library(packages_path: &Path, output: &Path) -> cu::Result<PackLibraryInfo> {
    let mut tar_builder = {
        let gz_encoder = GzEncoder::new(vec![], Compression::default());
        let mut builder = TarBuilder::new(gz_encoder);
        builder.mode(HeaderMode::Deterministic);
        builder.follow_symlinks(false);
        builder
    };
    let packages_path = packages_path.normalize()?;
    // lib
    {
        let source_path = packages_path.join("lib");
        let walk = IgnoreWalkBuilder::new(&source_path)
            .require_git(true)
            .add_custom_ignore_filename(".libpackignore")
            .build();
        add_to_tar(&mut tar_builder, walk, &source_path, Path::new("."))?;
    }

    // nnheaders
    {
        let source_path = packages_path.join("nnheaders");
        let walk = IgnoreWalkBuilder::new(&source_path)
            .require_git(true)
            .build();
        add_to_tar(&mut tar_builder, walk, &source_path, Path::new("nnheaders"))?;
    }
    // lib/Cargo.toml - need to make it a workspace
    {
        let manifest_path = packages_path.join("lib").join("Cargo.toml");
        let isolated_manifest = cu::check!(
            megaton_toolchain_build::create_isolated_cargo_manifest(
                &manifest_path,
                Some(r#"["macros"]"#)
            ),
            "failed to create isolated manifest for megaton library"
        )?;

        let bytes = isolated_manifest.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_path("Cargo.toml")?;
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar_builder.append(&header, bytes)?;
    }

    let gz_encoder = tar_builder.into_inner()?;
    let buffer = gz_encoder.finish()?;
    let sha256 = hash_sha256(&buffer);

    cu::fs::write(output, buffer)?;
    Ok(PackLibraryInfo { sha256 })
}
fn add_to_tar(
    tar_builder: &mut TarBuilder<GzEncoder<Vec<u8>>>,
    walk: IgnoreWalk,
    source_path: &Path,
    dest_path: &Path,
) -> cu::Result<()> {
    for entry in walk {
        let entry = entry?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let rel_path = dest_path.join(entry_path.try_to_rel_from(source_path));
        cu::ensure!(rel_path.is_relative(), "{}", rel_path.display())?;
        let mut file = cu::check!(
            File::open(entry_path),
            "failed to open '{}'",
            entry_path.display()
        )?;
        println!("cargo::rerun-if-changed={}", entry_path.as_utf8()?);
        tar_builder.append_file(&rel_path, &mut file)?;
    }
    Ok(())
}

fn hash_sha256(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for b in digest {
        // write to string cannot fail
        let _ = write!(hex, "{b:02x}");
    }
    hex
}
