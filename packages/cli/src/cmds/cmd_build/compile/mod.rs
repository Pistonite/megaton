// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use cu::pre::*;

use super::config::Flags;
use compile_db::CompileDB;

mod compile_db;
mod source;

/// Contextualizes compilations by associating a set of sources with certain build flags
pub struct CompileCtx {
    source_paths: Vec<PathBuf>,
    output_path: Arc<PathBuf>,
    flags: Arc<Flags>,
}

impl CompileCtx {
    /// Initialize a new compilation context
    pub fn new(source_paths: Vec<PathBuf>, output_path: PathBuf, flags: Flags) -> Self {
        Self {
            source_paths,
            output_path: Arc::new(output_path),
            flags: Arc::new(flags),
        }
    }
}

/// Compiles all sources found in multiple contexts, asyncronously
/// Will wait until all compilation jobs finish before returning
/// Handles loading, updating, and saving the compile db
/// Returns true if anything actually compiled
pub async fn compile_all(
    contexts: &[CompileCtx],
    compile_db_path: &Path,
) -> cu::Result<(bool, Vec<PathBuf>)> {
    // Get compile_db
    let mut compile_db = CompileDB::try_load_or_new(compile_db_path);

    if !compile_db.is_version_correct() {
        cu::info!("Compiler version has changed, recompiling");
        compile_db = CompileDB::new();
    }

    let pool = cu::co::pool(0);
    let mut handles = vec![];

    let progress = cu::progress("Compiling C/C++ objects").spawn();
    // Start compilation for all contexts
    for ctx in contexts {
        source::scan(&ctx.source_paths).for_each(|src| {
            cu::debug!("Scan: disovered source {}", src.path.display());
            let flags = ctx.flags.clone();
            let output_path = ctx.output_path.clone();
            let record = compile_db.find_record(src.pathhash);
            let progress = progress.clone();
            let handle = if let Some(record) = record {
                // Previous record found
                pool.spawn(src.compile(flags, output_path, Some(record.to_owned()), progress))
            } else {
                // No record of this file found
                pool.spawn(src.compile(flags, output_path, None, progress))
            };
            handles.push(handle);
        });
    }
    progress.done();

    let mut something_compiled = false;
    let mut objects = vec![];
    let mut errors = vec![];
    let mut set = cu::co::set(handles);
    while let Some(joined) = set.next().await {
        let res = joined.context("Failed to join handle")?;
        match res {
            Ok((did_compile, rec, o_path)) => {
                something_compiled |= did_compile;
                objects.push(o_path);
                compile_db.update(rec.source_hash, rec.clone());
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    compile_db.save(compile_db_path)?;

    if errors.is_empty() {
        Ok((something_compiled, objects))
    } else {
        let num = errors.len();
        let errorstring = errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        Err(cu::fmterr!(
            "Compilation failed with {num} errors: \n{errorstring}"
        ))
    }
}
