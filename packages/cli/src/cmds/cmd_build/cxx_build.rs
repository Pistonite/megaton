// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::path::Path;

use cu::fs::{walk, WalkEntry};

pub struct _BuildDependencyGraph {}

// returns a `Vec<String>` of source files that need to be compiled
pub fn source_scan(sources: &Vec<String>) -> Vec<String> {
    let mut files = Vec::<String>::new();
    for dir in sources {
        match walk(Path::new(dir)) {
            Ok(mut walk) => {
                while let Some(walk_result) = walk.next() {
                    match walk_result {
                        Ok(entry) => {
                            if is_source(&entry) && needs_recompile(&entry) {
                                match entry.file_name.into_string() {
                                    Ok(name) => {
                                        files.push(name);
                                    },
                                    Err(_) => {
                                        cu::warn!("Failed to read filename while walking {dir}");
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            cu::warn!("Failed to read entry while walking {dir}")
                        }
                    };
                }
            }
            Err(e) => {
                cu::warn!("{e}")
            }
        }
    }

    files
}

// Check if file extention is valid for accepted source types
fn is_source(file: &WalkEntry) -> bool {
    let name = file.file_name.clone().into_string().unwrap();

    match name.rsplit_once(".") {
        Some(split_name) => {
            split_name.1 == "cpp" || split_name.1 == "c" || split_name.1 == "s"
        },
        None => false,
    }
}

fn needs_recompile(_file: &WalkEntry) -> bool {
    // TODO: Calculuate if recomiple is required
    true
}
