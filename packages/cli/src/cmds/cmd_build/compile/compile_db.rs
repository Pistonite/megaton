use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use cu::pre::*;

use crate::env::environment;

#[derive(Serialize, Deserialize, Default)]
pub struct CompileDB {
    records: HashMap<usize, CompileRecord>,
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
    /// Generate a new compiledb with empty record table
    pub fn new() -> Self {
        let env = environment();
        Self {
            records: HashMap::default(),
            cc_version: env.cc_version().to_string(),
            cxx_version: env.cxx_version().to_string(),
            asm_version: env.asm_version().to_string(),
        }
    }

    /// Loads the compiledb, or makes a new one if one can't be found at the given path
    pub fn try_load_or_new(path: &Path) -> Self {
        if let Ok(file) = cu::fs::read(path) {
            let json = json::read::<CompileDB>(file.as_slice());
            match json {
                Ok(compdb) => compdb,
                Err(e) => {
                    cu::warn!(
                        "compdb at {} not readable due to error: {e}",
                        path.display()
                    );
                    cu::debug!("generating new compiledb");
                    Self::new()
                }
            }
        } else {
            cu::debug!("generating new compiledb");
            Self::new()
        }
    }

    /// Check if the recorded compiler versions are the same as in the environment
    pub fn version_is_correct(&self) -> bool {
        let env = environment();
        return env.cc_version() == self.cc_version
            && env.cxx_version() == self.cxx_version
            && env.asm_version() == self.asm_version;
    }

    /// Saves the compiledb to the disk, erroring if the path doesn't exist
    /// If the file already exists, it will be truncated and overwritten
    pub fn save(&self, path: &Path) -> cu::Result<()> {
        let file = std::fs::File::create(path)?;
        json::write_pretty(file, self)
    }

    pub fn find_record(&self, path_hash: usize) -> Option<&CompileRecord> {
        self.records.get(&path_hash)
    }

    pub fn update(&mut self, path_hash: usize, record: CompileRecord) {
        self.records.insert(path_hash, record);
    }

    // pub fn save_cc_json(&self, path: &Path) -> cu::Result<()> {
    //     let file = std::fs::File::create(path)?;
    //     let entries = self
    //         .commands
    //         .iter()
    //         .map(|cc| CCJsonEntry::from(cc))
    //         .collect::<Vec<CCJsonEntry>>();
    //
    //     json::write_pretty(file, &entries)
    // }
}

// #[derive(Serialize, Deserialize)]
// struct CCJsonEntry {
//     arguments: Vec<String>,
//     directory: String,
//     file: String,
// }
//
// impl From<&CompileRecord> for CCJsonEntry {
//     fn from(value: &CompileRecord) -> Self {
//         // Get rid of architecture-specific flags
//         let mut arguments = value
//             .args
//             .clone()
//             .into_iter()
//             .filter(|x| {
//                 let re = Regex::new(r"-mtune=.+|-march=.+|-mtp=.+").unwrap();
//                 !re.is_match(x)
//             })
//             .collect::<Vec<_>>();
//
//         arguments.extend(
//             environment()
//                 .dkp_headers()
//                 .into_iter()
//                 .map(|i| format!("-isystem {i}"))
//                 .collect::<Vec<_>>(),
//         );
//
//         let directory = PathBuf::from(".")
//             .canonicalize()
//             .unwrap()
//             .display()
//             .to_string();
//         let file = value.source.display().to_string();
//         Self {
//             arguments,
//             directory,
//             file,
//         }
//     }
// }
