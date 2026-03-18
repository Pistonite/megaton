// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

// use cu::pre::*;

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

/// Compiles all sources found in multiple contexts, asynchronously
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

    let mut total_tasks = 0;
    let progress = cu::progress("Compiling C/C++/S objects")
        .total(0)
        .eta(false)
        .percentage(false)
        .spawn();

    // Start compilation for all contexts
    for ctx in contexts {
        source::scan(&ctx.source_paths).for_each(|src| {
            cu::debug!("Scan: found source {}", src.path.display());
            let flags = ctx.flags.clone();
            let output_path = ctx.output_path.clone();
            let record = compile_db.find_record(src.pathhash).cloned();
            let progress = progress.clone();
            let handle =
                pool.spawn(async move { src.compile(flags, output_path, record, progress).await });
            handles.push(handle);
            total_tasks += 1;
        });
    }
    progress.set_total(total_tasks);

    let mut something_compiled = false;
    let mut objects = vec![];
    let mut num_errors = 0;
    let mut set = cu::co::set(handles);
    let mut completed_tasks = 0;
    while let Some(joined) = set.next().await {
        let res = joined?;
        match res {
            Ok((did_compile, rec, o_path)) => {
                something_compiled |= did_compile;
                objects.push(o_path);
                compile_db.update(rec.source_hash, rec.clone());
            }
            Err(_) => {
                num_errors += 1;
            }
        }
        completed_tasks += 1;
        cu::progress!(progress = completed_tasks);
    }

    compile_db.save(compile_db_path)?;
    drop(progress);

    if num_errors > 0 {
        // Just report how many errors we got. The tasks themselves will write to stderr as
        // compilation errors are encountered.
        cu::bail!("Compilation failed with {num_errors} errors");
    } else {
        Ok((something_compiled, objects))
    }
}
