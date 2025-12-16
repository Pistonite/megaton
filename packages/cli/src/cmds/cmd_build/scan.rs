// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

// This module handles scanning the mod/library source

// use cu::{Error, Result};
// use std::path::{PathBuf, Path};

// use super::{RustCrate, SourceFile};

// // Get every source file in the given directory, recursivly
// pub fn discover_source(dir: impl AsRef<Path>) -> Result<Vec<SourceFile>> {
//     // TODO: Implement
//     Ok(todo!())
// }

// // Find a rust crate in the given directory, if one exists
// pub fn discover_crates(dir: impl AsRef<Path>) -> Result<Vec<RustCrate>> {
//     // TODO: Implement
//     Ok(todo!())
// }
use std::ffi::OsStr;
use std::path::Path;

use cu::pre::*;

use super::RustCrate;
use crate::cmds::cmd_build::compile::{Lang, SourceFile};

// Get every source file in the given directory, recursivly
// Skips entrys with an unknown extension
// Warns if an entry cannot be read for some reason
pub fn discover_source(dir: &Path) -> cu::Result<Vec<SourceFile>> {
    let mut sources = Vec::new();
    let mut walk = cu::fs::walk(dir)?;
    while let Some(walk_result) = walk.next() {
        let entry = walk_result.context("failed to walk source directory")?;
        let path = entry.path();
        let lang = match path.extension().and_then(OsStr::to_str).unwrap_or_default() {
            "c" => Some(Lang::C),
            "cpp" | "c++" | "cc" | "cxx" => Some(Lang::Cpp),
            "s" | "asm" => Some(Lang::S),
            _ => None,
        };
        if let Some(lang) = lang {
            let source = SourceFile::new(lang, path).context("failed to create source object")?;
            sources.push(source);
        } else {
            cu::debug!(
                "Unrecognized extension: {}, skipping",
                path.to_str().unwrap_or("illegible filename")
            );
        }
    }
    Ok(sources)
}

// Find a rust crate in the given directory, if one exists
pub fn discover_crates(dir: impl AsRef<Path>) -> cu::Result<Vec<RustCrate>> {
    // TODO: Implement
    todo!()
}
