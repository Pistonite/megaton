// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use cu::pre::*;

use crate::env::environment;

type Records = HashMap<usize, CompileRecord>;

#[derive(Serialize, Deserialize, Default)]
pub struct CompileDB {
    records: Records,
    cc_version: String,
    cxx_version: String,
    asm_version: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CompileRecord {
    pub source_path: PathBuf,
    pub source_hash: usize,
    pub compiler: PathBuf,
    pub args: Vec<String>,
}

impl CompileDB {
    /// Loads the compiledb, or makes a new one if one can't be found at the given path
    pub fn try_load_or_new(path: &Path) -> Self {
        match Self::try_load(path) {
            Ok(db) => {
                cu::debug!("CompileDB loaded successfully");
                db
            }
            Err(e) => {
                cu::debug!("CompileDB failed to load: {e}");
                cu::info!("Generating a new CompileDB: {e}");
                Self::new()
            }
        }
    }

    /// Attempt to load the compiledb from the given path
    /// Errors if it can't be read, or if json parsing fails
    pub fn try_load(path: &Path) -> cu::Result<Self> {
        if let Ok(file) = cu::fs::read(path) {
            json::read::<CompileDB>(file.as_slice())
        } else {
            Err(cu::fmterr!("CompileDB not found at {}", path.display()))
        }
    }

    /// Generate a new compiledb with empty record table
    pub fn new() -> Self {
        let env = environment();
        Self {
            records: Records::default(),
            cc_version: env.cc_version().to_string(),
            cxx_version: env.cxx_version().to_string(),
            asm_version: env.asm_version().to_string(),
        }
    }

    /// Check if the recorded compiler versions are the same as in the environment
    pub fn is_version_correct(&self) -> bool {
        let env = environment();
        env.cc_version() == self.cc_version
            && env.cxx_version() == self.cxx_version
            && env.asm_version() == self.asm_version
    }

    /// Saves the compiledb to the disk, erroring if the path doesn't exist
    /// If the file already exists, it will be truncated and overwritten
    pub fn save(&self, path: &Path) -> cu::Result<()> {
        cu::fs::write_json_pretty(path, self)
    }

    pub fn find_record(&self, path_hash: usize) -> Option<&CompileRecord> {
        self.records.get(&path_hash)
    }

    pub fn update(&mut self, path_hash: usize, record: CompileRecord) {
        self.records.insert(path_hash, record);
    }
}
