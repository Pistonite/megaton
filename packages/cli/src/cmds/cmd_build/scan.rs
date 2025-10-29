// This module handles scanning the mod/library source

use cu::{debug, fs::walk, Result};
use std::path::Path;
use std::ffi::OsStr;

use super::{RustCrate, SourceFile, Lang};

// Get every source file in the given directory, recursivly
// Skips entrys with an unknown extension
// Warns if an entry cannot be read for some reason
pub fn discover_source(dir: impl AsRef<Path>) -> Result<Vec<SourceFile>> {
    let mut sources = Vec::new();
    let mut walk = walk(Path::new(dir.as_ref()))?;
    while let Some(walk_result) = walk.next() {
        match walk_result {
            Ok(entry) => {
                let path = entry.path();
                let lang = match path.extension().and_then(OsStr::to_str).unwrap_or_default() {
                    "c" => Some(Lang::C),
                    "cpp" | "c++" | "cc" | "cxx" => Some(Lang::Cpp),
                    "s" | "asm" => Some(Lang::S),
                    _ => None
                };
                if let Some(lang) = lang {
                    let source = SourceFile::new(lang, path);
                    sources.push(source);
                } else {
                    cu::debug!("Unrecognized extension: {}, skipping", path.to_str().unwrap_or("illegible filename"));
                }
            }
            Err(e) => {
                cu::warn!("{e}");
            }
        };
    }
    Ok(sources)
}

// Find a rust crate in the given directory, if one exists
pub fn discover_crates(dir: impl AsRef<Path>) -> Result<Vec<RustCrate>> {
    // TODO: Implement
    Ok(todo!())
}
