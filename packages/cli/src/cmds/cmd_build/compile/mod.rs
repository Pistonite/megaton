use std::{ path::{Path, PathBuf},
    sync::Arc,
};

use futures::future::{try_join_all};

use super::config::Flags;
use compile_db::{CompileDB, CompileRecord};

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
pub async fn compile_all(contexts: &[CompileCtx], compile_db_path: &Path) -> cu::Result<bool> {
    // Get compile_db
    let mut compile_db = CompileDB::load(compile_db_path);

    if !compile_db.version_is_correct() {
        compile_db = CompileDB::new();
    }

    let pool = cu::co::pool(0);
    let mut handles: Vec<cu::co::Handle<cu::Result<(bool, CompileRecord)>>> = vec![];

    // Start compilation for all contexts
    for ctx in contexts {
        source::scan(&ctx.source_paths)
            .map(|src| {
                let flags = ctx.flags.clone();
                let output_path = ctx.output_path.clone();
                let record = compile_db.find_record(src.pathhash);
                if let Some(record) = record {
                    // Previous record found
                    pool.spawn(src.compile(flags, output_path, Some(record.to_owned())))
                } else {
                    // No record of this file found
                    pool.spawn(src.compile(flags, output_path, None))
                }
            })
            .for_each(|handle| handles.push(handle));
    }

    let mut something_compiled = false;

    // Join all compilation jobs
    let results = try_join_all(handles.into_iter().map(|h| h.co_join())).await?;
    results.iter().for_each(|rec| match rec {
        Ok((did_compile, rec)) => {
            something_compiled |= did_compile;
            compile_db.update(rec.source_hash, rec.clone());
        },
        Err(_) => {
            // TODO: Fix so this isnt a result. Any errors should be
            // returned earlier in try_join_all
            panic!("Error while parsing compile job result (idk why this would happen)");
        },
    });

    compile_db.save(compile_db_path)?;
    Ok(something_compiled)
}
