// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors
use std::path::PathBuf;
use std::process::ExitCode;

use cu::pre::*;

fn main() {
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
    let commit_hash = megaton_packaging::get_commit()?;
    println!("cargo::rustc-env=MEGATON_COMMIT={commit_hash}");
    Ok(())
}
