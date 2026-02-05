use crate::cmds::cmd_build::{
    compile::source_file::discover_source_files, compile_db::CompileDB, config::Flags,
};
use futures::future::join_all;
use std::path::PathBuf;

pub async fn run_compilation(
    prev_compile_db: CompileDB,
    sources: Vec<PathBuf>,
    flags: &Flags,
    output_path: &PathBuf,
    includes: Vec<String>,
) -> cu::Result<CompileDB> {
    let sources = sources
        .iter()
        .flat_map(|source_directory| {
            let discovered_source_files = discover_source_files(source_directory);
            let _ = discovered_source_files.as_ref().inspect_err(|e| {
                cu::error!(
                    "Failed to discover sources in {:?}: {}",
                    source_directory,
                    e
                )
            });
            discovered_source_files.unwrap_or_default()
        })
        .collect::<Vec<_>>();

    let sources = sources.iter().map(|src| (src, src.build_compile_command(&output_path, flags, &includes))).collect::<Vec<_>>();
    let mut compilation_futures = vec![];
    sources.
        iter().for_each(|(src, compile_command)| { 
        let old_record = prev_compile_db.find_record(src.pathhash);
        if src.need_recompile(old_record, &output_path, &compile_command).unwrap_or(true) {
            let child = compile_command.execute();
            compilation_futures.push((child, compile_command));
        }
    });

    // await compilation of all source files
    let mut compilation_results= vec![];
    for (future, cc) in compilation_futures {
        let child = future.await?;
        let result = child.co_wait_nz().await;
        if result.is_ok(){
            compilation_results.push(cc)
        }
    }

    Ok(CompileDB::new(compilation_results))
}
