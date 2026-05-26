// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors
use std::path::Path;
use std::process::ExitCode;

use cu::pre::*;

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
    let crate_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_output_path = crate_path.join("libmegaton.tar.gz");

    let commit_hash = megaton_cli_build::get_commit()?;
    println!("cargo::rustc-env=MEGATON_COMMIT={commit_hash}");
    let packages_path = crate_path.parent_abs()?;
    let info = cu::check!(megaton_cli_build::pack_library(&packages_path, &lib_output_path), "failed to pack library")?;
    println!("cargo::rustc-env=MEGATON_LIB_SHA256={}",info.sha256);

    Ok(())
}
