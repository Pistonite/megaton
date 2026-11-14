// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors
use std::process::ExitCode;

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
    // support reading from COMMIT file for docker builds
    let md = cu::env_var("CARGO_MANIFEST_DIR").unwrap_or_default();
    if md.is_empty() {
        cu::bail!("failed to get CARGO_MANIFEST_DIR, please run the packaging script with cargo");
    }
    let commit_file_path = cu::path!(&(&md) / "COMMIT");
    if commit_file_path.exists() {
        let commit = cu::fs::read_string(commit_file_path)?;
        let commit = commit.trim().to_lowercase();
        // todo: when it becomes sha256 by default this might need to change
        if commit.len() != 40 || commit.chars().any(|c| 
            !c.is_ascii_hexdigit()
        ) {
            cu::bail!("{commit} is not a valid git commit hash");
        }
        println!("cargo::rustc-env=MEGATON_COMMIT={commit}");
        return Ok(());
    }
    let commit_hash = megaton_packaging::get_commit()?;
    println!("cargo::rustc-env=MEGATON_COMMIT={commit_hash}");
    Ok(())
}
