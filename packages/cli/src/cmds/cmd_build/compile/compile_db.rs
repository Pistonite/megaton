// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use cu::pre::*;

use crate::env::environment;

type Records = BTreeMap<usize, CompileRecord>;

#[derive(Serialize, Deserialize, Default)]
pub struct CompileDB {
    records: Records,
    cc_version: String,
    cxx_version: String,
    asm_version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CompileRecord {
    pub source_path: PathBuf,
    pub compiler: PathBuf,
    pub args: Vec<String>,
    pub o_path: PathBuf, // -o argument already in args
    pub d_path: PathBuf,
}

impl CompileDB {
    /// Loads the compiledb, or makes a new one if one can't be found at the given path
    pub fn try_load_or_new(path: &Path) -> Self {
        match Self::try_load(path) {
            Ok(db) => {
                cu::debug!("Compile: loaded compiledb");
                db
            }
            Err(e) => {
                cu::debug!("Compile: compiledb failed to load: {e:?}, creating a new one");
                Self::new()
            }
        }
    }

    /// Attempt to load the compiledb from the given path
    /// Errors if it can't be read, or if json parsing fails
    pub fn try_load(path: &Path) -> cu::Result<Self> {
        let file = cu::fs::read(path)?;
        json::read::<CompileDB>(file.as_slice())
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

impl CompileRecord {
    pub async fn compile(&self, parent_progress: Arc<cu::ProgressBar>) -> cu::Result<()> {
        let start_time = cu::fs::Time::now();

        let progress = parent_progress
            .child(format!("{}", self.source_path.try_to_rel().display()))
            .spawn();

        self.compiler
            .command()
            .stdout(cu::lv::T)
            .stderr(cu::lv::E)
            .stdin_null()
            .args(&self.args)
            .co_wait_nz()
            .await?;

        progress.done();
        cu::debug!("Compile: compiled object {}", self.o_path.display());

        cu::fs::set_mtime(&self.source_path, start_time)?;
        cu::fs::set_mtime(&self.o_path, start_time)?;
        if self.d_path.exists() {
            cu::fs::set_mtime(&self.d_path, start_time)?;
        }

        Ok(())
    }
}

// Ordered so the vec order also stays the same
type CompileCommandsContainer = BTreeSet<CompileCommandsEntry>;

pub struct CompileCommands {
    entries: CompileCommandsContainer,
}

// TODO: this is similar to CompileDB, maybe generics/traits could make these share some implementation?
impl CompileCommands {
    pub fn try_load_or_new(path: &Path) -> Self {
        match Self::try_load(path) {
            Ok(db) => {
                cu::debug!("Compile: loaded compile_commands.json");
                db
            }
            Err(e) => {
                cu::debug!(
                    "Compile: compile_commands.json failed to load: {e:?}, creating a new one"
                );
                Self {
                    entries: CompileCommandsContainer::new(),
                }
            }
        }
    }

    fn try_load(path: &Path) -> cu::Result<Self> {
        let file = cu::fs::read(path)?;
        let vec = json::read::<Vec<CompileCommandsEntry>>(file.as_slice())?;
        let container = CompileCommandsContainer::from_iter(vec);
        Ok(Self { entries: container })
    }

    pub fn save(self, path: &Path) -> cu::Result<()> {
        let vec: Vec<CompileCommandsEntry> = self.entries.into_iter().collect();
        cu::fs::write_json_pretty(path, &vec)
    }

    pub fn update(&mut self, entry: CompileCommandsEntry) {
        self.entries.insert(entry);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompileCommandsEntry {
    arguments: Vec<String>,
    directory: String,
    file: String,
}

impl From<&CompileRecord> for CompileCommandsEntry {
    fn from(value: &CompileRecord) -> Self {
        let env = environment().dkp_includes();
        let arguments = value
            .clone()
            .args
            .into_iter()
            .filter_map(|arg| {
                if let Some(path) = arg.strip_prefix("-I") {
                    let path = &String::from(path);
                    if env.contains(path) {
                        let thing = format!("-isystem{}", path);
                        Some(thing)
                    } else {
                        Some(arg)
                    }
                } else if arg.contains("-mtune=")
                    || arg.contains("-march=")
                    || arg.contains("-mtp=")
                {
                    None
                } else {
                    Some(arg)
                }
            })
            .collect::<Vec<_>>();

        // This should never be able to panic
        let directory = PathBuf::from(".")
            .canonicalize()
            .unwrap()
            .into_utf8()
            .unwrap();

        // If the path in the compiledb is valid, we can assume that calling into_utf8 will succeed
        let file = value.source_path.clone().into_utf8().unwrap();
        Self {
            arguments,
            directory,
            file,
        }
    }
}
