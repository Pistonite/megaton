// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

//! Manages locally user (sparse-)checked-out megaton repo
//!
//! This is for copying megaton lib into target/megaton/lib
//! when building

use std::path::Path;

use cu::pre::*;

use crate::env;

pub struct MegatonRepo {

}

/// Get the locally checked-out megaton repo in the user's cache.
/// Checking it out if not already.
pub fn megaton_repo() -> cu::Result<MegatonRepo> {
    let repo_root = env::environment().repo("megaton");

    if let Err(e) = check_megaton_repo_up_to_date(&repo_root) {
        cu::debug!("repo check failed: {e}, reclone needed");
        cu::fs::make_dir_empty(&repo_root).context("failed to clean megaton repo while attempting clone")?;
    }

    todo!()

}

fn check_megaton_repo_up_to_date(path: &Path) -> cu::Result<()> {
    let (child, commit_hash) = cu::which("git")?.command()
        .add(cu::args!["-C", path, "rev-parse", "HEAD"])
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let commit_hash = commit_hash.join()??;
    let commit_hash = commit_hash.trim();

    if commit_hash != env::commit() {
        cu::bail!("cached megaton repo commit hash mismatch!");
    }

    Ok(())
}
