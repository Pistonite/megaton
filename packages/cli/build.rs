// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors
use std::io::Write;
use std::path::PathBuf;
use std::{fs::File, path::Path};

use cu::pre::*;
use flate2::{Compression, write::GzEncoder};
use sha2::{Digest, Sha256};
use tar::{Builder as TarBuilder, HeaderMode};

fn main() -> cu::Result<()> {
    let (child, commit_hash) = cu::which("git")?
        .command()
        .args(["rev-parse", "HEAD"])
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let commit_hash = commit_hash.join()??;
    let commit_hash = commit_hash.trim();
    println!("cargo::rustc-env=MEGATON_COMMIT={commit_hash}");

    let lib_path = make_lib_targz()?;
    gen_lib_hash(&lib_path)?;
    Ok(())
}

/// Pack the megaton lib sources into libmegaton.tar.gz
fn make_lib_targz() -> cu::Result<PathBuf> {
    let crate_path = PathBuf::from(cu::env_var("CARGO_MANIFEST_DIR")?);
    let path = crate_path.join("libmegaton.tar.gz");

    let mut tar_builder = {
        let file = cu::fs::writer(&path)?;
        let gz_encoder = GzEncoder::new(file, Compression::default());
        let mut builder = TarBuilder::new(gz_encoder);
        builder.mode(HeaderMode::Deterministic);
        builder.follow_symlinks(false);
        builder
    };

    let lib_path = {
        let mut path = crate_path.parent_abs()?;
        path.push("lib");
        path
    };
    let mut walk = cu::fs::walk(&lib_path)?;
    while let Some(entry) = walk.next() {
        let entry = entry?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let rel_path = entry_path.try_to_rel_from(&lib_path);
        cu::ensure!(
            rel_path.is_relative(),
            "not relative: {}",
            rel_path.display()
        )?;
        let mut file = cu::check!(
            File::open(&entry_path),
            "failed to open '{}'",
            entry_path.display()
        )?;
        println!("cargo::rerun-if-changed={}", entry_path.as_utf8()?);
        tar_builder.append_file(&rel_path, &mut file)?;
    }
    tar_builder.into_inner()?.finish()?.flush()?;
    Ok(path)
}

fn gen_lib_hash(lib: &Path) -> cu::Result<()> {
    let lib_bytes = cu::fs::read(lib)?;
    let hash = Sha256::digest(lib_bytes);
    let hashfile_path =
        PathBuf::from(cu::env_var("CARGO_MANIFEST_DIR")?).join("libmegaton_sha256sum");
    cu::fs::write(hashfile_path, hash)?;
    Ok(())
}
