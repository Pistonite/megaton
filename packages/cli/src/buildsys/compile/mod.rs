// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use compile_db::{CompileCommands, CompileCommandsEntry, CompileDB};
use source::SourceStatus;

use crate::config::Flags;

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
    if !compile_db.is_version_correct() {
        cu::info!("Compiler version has changed, recompiling");
        compile_db = CompileDB::new();
    }

    let mut compile_commands = CompileCommands::try_load_or_new(compile_commands_path);

    // Even if an object doesn't need compiled, we still need to pass back its path since this is
    // how the linker knows what objects need linked
    let mut objects = vec![];
    let mut handles = vec![];
    let pool = cu::co::pool(0);
    let mut total_tasks = 0;

    let mut progress_bar = None;

    // Configure all sources and start compile tasks
    for ctx in contexts {
        for src in source::scan(&ctx.source_paths) {
            let record = compile_db.find_record(src.pathhash);
            let source_hash = src.pathhash;
            match src.configure_compilation(&ctx.flags, &ctx.output_path, record)? {
                SourceStatus::UpToDate(object) => {
                    if !configure_only {
                        cu::debug!("Compile: object up to date {}", &object.display());
                    }
                    // Record must exist if object is up to date
                    let old_rec = record.unwrap();
                    let entry = CompileCommandsEntry::from(old_rec);
                    compile_commands.update(entry);
                    objects.push(object);
                }
                SourceStatus::CompileNeeded(compile_record) => {
                    objects.push(compile_record.o_path.clone());
                    let entry = CompileCommandsEntry::from(&compile_record);
                    compile_commands.update(entry);

                    if !configure_only {
                        if progress_bar.is_none() {
                            progress_bar = Some(
                                cu::progress("Compile")
                                    .total(0)
                                    .eta(false)
                                    .percentage(false)
                                    .spawn(),
                            );
                        }
                        let parent_progress = progress_bar.clone().unwrap().clone();
                        compile_db.update(source_hash, compile_record.clone());
                        let handle = pool
                            .spawn(async move { compile_record.compile(parent_progress).await });
                        handles.push(handle);
                        total_tasks += 1;
                    }
                }
            }
        }
    }

    if handles.is_empty() {
        compile_commands.save(compile_commands_path)?;
        cu::debug!("Compile: updated compile_commands.json");
        return Ok((false, objects));
    }

    let progress_bar = progress_bar.unwrap();

    progress_bar.set_total(total_tasks);

    let mut completed_tasks = 0;
    let mut num_errors = 0;
    let mut set = cu::co::set(handles);
    while let Some(joined) = set.next().await {
        if joined?.is_err() {
            num_errors += 1;
        }
        completed_tasks += 1;
        cu::progress!(progress_bar = completed_tasks);
    }

    progress_bar.done();
    compile_commands.save(compile_commands_path)?;
    compile_db.save(compile_db_path)?;

    if num_errors > 0 {
        // Just report how many errors we got. The tasks themselves will write to stderr as
        // compilation errors are encountered.
        cu::bail!("Compilation failed with {num_errors} errors");
    } else {
        Ok((true, objects))
    }
}
