// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use cu::pre::*;

use super::config::Flags;
use crate::cmds::cmd_build::compile::{
    compile_db::try_load_or_new_compile_commands, source::SourceStatus,
};
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
    compile_commands_path: &Path,
    configure_only: bool,
) -> cu::Result<(bool, Vec<PathBuf>)> {
    // Get compile_db
    let mut compile_db = CompileDB::try_load_or_new(compile_db_path);

    let mut compile_commands = try_load_or_new_compile_commands(compile_commands_path);

    if !compile_db.is_version_correct() {
        cu::info!("Compiler version has changed, recompiling");
        compile_db = CompileDB::new();
    }

    // Even if an object doesn't need compiled, we still need to pass back its path since this is
    // how the linker knows what objects need linked
    let mut objects = vec![];
    let mut handles = vec![];
    let pool = cu::co::pool(0);

    let mut total_tasks = 0;
    let progress = cu::progress("C/C++/S")
        .total(handles.len())
        .eta(false)
        .percentage(false)
        .spawn();
    if configure_only {
        cu::progress!(progress, "configuring compile_commands.json");
    } else {
        cu::progress!(progress, "compiling sources");
    }

    // Configure all sources and start compile tasks
    for ctx in contexts {
        for src in source::scan(&ctx.source_paths) {
            let record = compile_db.find_record(src.pathhash);
            match src.configure_compilation(&ctx.flags, &ctx.output_path, record)? {
                SourceStatus::UpToDate(object) => {
                    if !configure_only {
                        cu::debug!("Compile: object up to date {}", &object.display());
                    }
                    objects.push(object);
                }
                SourceStatus::CompileNeeded(compile_record) => {
                    objects.push(compile_record.o_path.clone());
                    if !configure_only {
                        let parent_progress = progress.clone();
                        compile_db.update(compile_record.source_hash, compile_record.clone());
                        let handle = pool
                            .spawn(async move { compile_record.compile(parent_progress).await });
                        handles.push(handle);
                        total_tasks += 1;
                    }
                }
            }
        }
    }

    if configure_only || handles.is_empty() {
        progress.done();
        cu::fs::write_json_pretty(compile_commands_path, &compile_commands)?;
        return Ok((false, objects));
    }

    progress.set_total(total_tasks);

    let mut completed_tasks = 0;
    let mut num_errors = 0;
    let mut set = cu::co::set(handles);
    while let Some(joined) = set.next().await {
        if joined?.is_err() {
            num_errors += 1;
        }
        completed_tasks += 1;
        cu::progress!(progress = completed_tasks);
    }

    progress.done();
    cu::fs::write_json_pretty(compile_commands_path, &compile_commands)?;
    compile_db.save(compile_db_path)?;

    if num_errors > 0 {
        // Just report how many errors we got. The tasks themselves will write to stderr as
        // compilation errors are encountered.
        cu::bail!("Compilation failed with {num_errors} errors");
    } else {
        Ok((true, objects))
    }
}
