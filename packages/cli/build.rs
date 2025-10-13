// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use cu::pre::*;

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
    Ok(())
}
