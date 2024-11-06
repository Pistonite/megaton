//! Utilities for detecting and processing source file types
use rustc_hash::FxHasher;

use crate::prelude::*;

use std::{hash::{Hash, Hasher}, path::Path};

use crate::system::Error;

/// Source file types
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum SourceType {
    /// C source file
    C,
    /// C++ source file
    Cpp,
    /// Assembly source file
    S,
}

impl SourceType {
    /// Parse
    pub fn parse_path(source: &str) -> Option<(SourceType, &str, &str)> {
        let dot = source.rfind('.').unwrap_or(source.len());
        let ext = &source[dot..];
        let source_type = SourceType::from_ext(ext)?;
        let slash = source.rfind(|c| c == '/' || c == '\\').unwrap_or(0);
        let base = &source[slash + 1..dot];
        if base.is_empty() {
            return None;
        }
        Some((source_type, base, ext))
    }
    /// Get source type from file extension
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "c" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "c++" => Some(Self::Cpp),
            "s" | "asm" => Some(Self::S),
            _ => None,
        }
    }
}

pub struct SourceFile {
    /// Type of the source file (used to determine which build rule to run)
    pub typ: SourceType,

    /// Full path of the source file
    pub path: String,

    /// Name of the object file produced in the format of `{base}-{hash}{ext}`.
    ///
    /// By adding .o or .d to this name, we can get the object file or dependency file.
    pub name_hash: String,
}

impl SourceFile {
    /// Create source file property from path. Path should be full (absolute) path
    ///
    /// Return None if the file is not a source file
    pub fn from_path(source: &Path) -> Result<Option<Self>, Error> {
        let path = source.to_utf8()?;

        // find extension without dot and get source type
        let dot = match path.rfind('.') {
            Some(dot) => dot,
            None => return Ok(None),
        };
        let ext = &path[dot+1..];
        let typ = match SourceType::from_ext(ext) {
            Some(typ) => typ,
            None => return Ok(None),
        };

        // get the base name of file
        let slash = path.rfind(|c| c == '/' || c == '\\').unwrap_or(0);
        let base = &path[slash + 1..dot];
        if base.is_empty() {
            return Ok(None);
        }

        // hash the full path
        let mut hasher = FxHasher::default();
        source.hash(&mut hasher);
        let hash = hasher.finish();

        let object_name = format!("{}-{:016x}.{}", base, hash, ext);

        Ok(Some(Self {
            typ,
            path,
            name_hash: object_name,
        }))
    }

}
