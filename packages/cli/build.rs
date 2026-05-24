// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors
use std::fs::File;
use std::path::Path;
use std::process::ExitCode;

use cu::pre::*;
use flate2::{Compression, write::GzEncoder};
use ignore::{Walk as IgnoreWalk, WalkBuilder as IgnoreWalkBuilder};
use sha2::{Digest, Sha256};
use tar::{Builder as TarBuilder, HeaderMode};

fn main() -> ExitCode {
    match main_internal() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            for l in format!("{e:?}").lines() {
                println!("cargo::error={l}");
            }
            ExitCode::FAILURE
        }
    }
}

fn main_internal() -> cu::Result<()> {
    cu::check!(export_commit_env(), "failed to export commit env")?;

    cu::check!(make_lib_targz(), "failed to make lib package")?;
    Ok(())
}

fn export_commit_env() -> cu::Result<()> {
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
    Ok(())
}

/// Pack the megaton lib sources into libmegaton.tar.gz
fn make_lib_targz() -> cu::Result<()> {
    let crate_path = crate_path();
    let output_path = crate_path.join("libmegaton.tar.gz");

    let mut tar_builder = {
        // let file = cu::fs::writer(&path)?;
        let gz_encoder = GzEncoder::new(vec![], Compression::default());
        let mut builder = TarBuilder::new(gz_encoder);
        builder.mode(HeaderMode::Deterministic);
        builder.follow_symlinks(false);
        builder
    };

    let packages_path = crate_path.parent_abs()?;
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
        let mut cargo_toml = cu::fs::read_string(packages_path.join("lib").join("Cargo.toml"))?;
        cargo_toml.push_str(
            r##"
[workspace]
resolver = 2
members = [ "macros" ]
        "##,
        );
        let bytes = cargo_toml.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_path("Cargo.toml")?;
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar_builder.append(&header, bytes)?;
    }

    let gz_encoder = tar_builder.into_inner()?;
    let buffer = gz_encoder.finish()?;
    export_lib_hash(&buffer);

    cu::fs::write(output_path, buffer)?;

    Ok(())
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

fn export_lib_hash(bytes: &[u8]) {
    use std::fmt::Write as _;

    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for b in digest {
        // write to string cannot fail
        let _ = write!(hex, "{b:02x}");
    }
    println!("cargo::rustc-env=MEGATON_LIB_SHA256={hex}");
}

fn crate_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
