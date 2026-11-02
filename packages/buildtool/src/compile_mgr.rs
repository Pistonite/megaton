use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use cu::pre::*;

use megaton_config::config_file::{BuildConfig, Config};

use crate::compile_commands::{CompileCommand, CompileDatabase};
use crate::source_type::SourceType;
use crate::path_hash;

pub struct CompileMgr {
    /// Only generate configuration, don't compile/link
    configure_only: bool,
    build_config: BuildConfig,
    project_root: PathBuf,
    /// Directory to save as the build directory in compile database.
    ///
    /// In reality this doesn't matter because all paths will be absolute
    build_dir: String,
    /// Directory to emit objects
    object_dir: PathBuf,
    /// compile_commands.json to save compile commands to.
    /// The flags will be modified to work with clangd. Existing
    /// sources in this file will not be replaced, so this can be
    /// merged with other compile_commands.json if the LSP setup
    /// only allows one.
    clangd_compile_commands_json_path: PathBuf,
    /// Megaton's own compile_commands.json
    compile_database: CompileDatabase,
    /// Cache of checked file times
    mtime_cache: BTreeMap<String, Option<cu::fs::Time>>,
    /// Progress bar
    bar: LazyLock<Arc<cu::ProgressBar>>,
    /// Compiler process pool
    pool: cu::co::Pool,
    /// Compiler process handles
    handles: Vec<cu::co::Handle<cu::Result<String>>>,
    /// Output objects that need to be linked
    objects: Vec<String>,
    /// If no more compile can start
    objects_submitted: bool,
}

impl CompileMgr {
    #[cu::context("failed to create compile manager")]
    pub fn new(
        configure_only: bool,
        config: &Config,
    ) -> cu::Result<Self> {
        let build_config = config.build_config()?.clone();

        let compile_database = CompileDatabase::load(
            &config.project.output_root.join("compile_cache.json")
        )?;

        Ok(Self {
            configure_only,
            build_config,
            project_root:  config.project.root.clone(),
            build_dir: config.project.output_root.as_utf8()?.to_string(),
            object_dir: config.project.out_objects.clone(),
            clangd_compile_commands_json_path: config.module.compile_commands.clone(),
            compile_database,
            mtime_cache: Default::default(),
            bar:  LazyLock::new(|| { cu::progress("compiling").total(0).percentage(false).eta(false).spawn()
            }),
            pool: cu::co::pool(-1),
            handles: Default::default(),
            objects: Default::default(),
            objects_submitted: false
        })
    }

    pub fn build_config(&self) -> &BuildConfig {
        &self.build_config
    }
}

impl CompileMgr {
    /// Start a compiler process in the pool to compile the input file
    /// if needed, no-op if the input file is not a source file
    pub fn compile(&mut self, file: &Path, ctrlc: cu::CtrlcSignal
    ) -> cu::Result<()> {
        let file_rel = file.try_to_rel_from(&self.project_root);
        let file_display = file_rel.to_string_lossy();
        self.compile_imp(file, &file_display, ctrlc)
    }

    #[cu::context(pre, format("failed to configure file for compilation: {file_display}"))]
    fn compile_imp(&mut self, file: &Path, file_display: &str,
        ctrlc: cu::CtrlcSignal
    ) -> cu::Result<()> {
        if self.objects_submitted {
            cu::bail!("unexpected: cannot compile another source after linking has started");
        }
        let Some(ext) = file.extension() else {
            cu::debug!("not a source file (no extension): {file_display}");
            return Ok(());
        };
        let Some(source_type) = SourceType::from_extension(ext) else {
            cu::debug!("not a source file (unrecognized extension): {file_display}");
            return Ok(());
        };

        let out_base_path = path_hash::hashed_output_path(file, &self.object_dir)?;
        let obj_path = out_base_path.with_added_extension(".o");
        let dep_path = out_base_path.with_added_extension(".d");

        // create compile command
        let compiler = source_type.get_compiler(&self.build_config);
        // actual flags to invoke the compiler with
        let mut invocation_flags = vec![
            "-MMD".to_string(),
            "-MP".to_string(),
            "-MF".to_string(),
            dep_path.as_utf8()?.to_string()
        ];
        let color_flag = if cu::lv::color_enabled() {
            "never"
        } else {
            "always"
        };
        invocation_flags.push(format!("-fdiagnostics-color={color_flag}"));
        // flags to save to compile cache
        let mut arguments = vec![compiler.to_string()];
        // flags to save to clangd-compatible compile cache
        let mut clangd_compat_arguments = vec![compiler.to_string()];

        let flags = source_type.get_flags(&self.build_config);
        invocation_flags.extend_from_slice(flags);
        arguments.extend_from_slice(flags);
        clangd_compat_arguments.extend_from_slice(
            source_type.get_clangd_compat_flags(&self.build_config)
        );

        let obj_str = obj_path.as_utf8()?.to_string();
        let src_str = file.as_utf8()?.to_string();

        for arg in ["-c", "-o", &obj_str, &src_str ] {
            invocation_flags.push(arg.to_string());
            arguments.push(arg.to_string());
            clangd_compat_arguments.push(arg.to_string());
        }

        let command = CompileCommand { 
            directory: self.build_dir.clone(),
            file: src_str,
            output: obj_str.clone(),
            arguments,
            clangd_compat_arguments,
            command: None
        };

        if self.configure_only {
            self.compile_database.update(Cow::Owned(command));
            cu::debug!("(configure-only) updated compile database for {}", obj_path.display());
            return Ok(());
        }

        let is_up_to_date = check_up_to_date(
            source_type, file, &obj_path, &dep_path, command, &mut self.compile_database
            , &mut self.mtime_cache
        );

        if is_up_to_date {
            self.objects.push(obj_str);
            return Ok(());
        }

        let file_display = file_display.to_string();
        let compiler = PathBuf::from(compiler);
        let bar = Arc::clone(&self.bar);
        let handle = self.pool.spawn(async move {
            let (mut child, stdout, stderr) = compiler.command().args(invocation_flags)
            .stdout(cu::pio::buffer())
                .stderr(cu::pio::buffer())
                .stdin_null()
                .co_spawn().await?;
            let result = loop {
                match child.co_wait_timeout(Duration::from_millis(100)).await {
                    Err(e) => {
                        cu::debug!("killing compiler process on wait error!");
                        if let Err(e) = child.co_kill().await {
                            cu::debug!("error killing compiler process: {e:?}");
                        }
                        break Err(e);
                    }
                    Ok(None) => {
                        if ctrlc.signaled() {
                            cu::debug!("killing compiler process on ctrl-c!");
                            if let Err(e) = child.co_kill().await {
                                cu::debug!("error killing compiler process: {e:?}");
                            }
                            break Err(cu::fmterr!("interrupted"))
                        }
                    }
                    Ok(Some(exited)) => {
                        break if exited.success() {
                            Ok(())
                        } else {
                            Err(cu::fmterr!("compiler exited with {exited}"))
                        }
                    }
                }
            };
            // always print stderr since there might be warnings
            if let Ok(stderr) = stderr.co_join().await.flatten() {
                cu::print!("{}", String::from_utf8_lossy(&stderr));
            }
            if let Err(e) = result {
                if let Ok(stdout) = stdout.co_join().await.flatten() {
                    cu::print!("{}", String::from_utf8_lossy(&stdout));
                }
                return Err(e);
            }

            cu::progress!(bar += 1, "{file_display}");
            cu::Ok(obj_str)
        });

        self.handles.push(handle);
        Ok(())
    }

    /// Wait for all objects to be compiled
    pub async fn finish_compiling(&mut self) -> cu::Result<()> {
        if self.handles.is_empty() && self.objects.is_empty() {
            cu::bail!("no source files found to compile!");
        }
        self.objects_submitted = true;
        let count = self.handles.len(); 
        self.bar.set_total(count as u64);
        self.objects.reserve(count);

        let mut set = cu::co::set(std::mem::take(&mut self.handles));
        let mut error_count = 0;
        while let Some(result) = set.next().await {
            match result.flatten() {
                Err(e) => {
                    error_count += 1;
                    cu::debug!("error when compiling: {e:?}");
                }
                Ok(object) => {
                    self.objects.push(object);
                }
            }
        }

        if error_count > 0 {
            cu::bail!("{error_count} objects failed to compile");
        }

        Ok(())
    }

    pub fn save_compile_database(&self) -> cu::Result<()> {
        self.compile_database.save_clangd_compat_merge_with(&self.clangd_compile_commands_json_path)?;
        self.compile_database.save()?;
        Ok(())
    }

    pub fn into_objects(self) -> Vec<String> {
        self.objects
    }
}

/// Check if a compilation edge is up-to-date
fn check_up_to_date(
    source_type: SourceType,
    src_path: &Path,
    obj_path: &Path,
    dep_path: &Path,
    command: CompileCommand,
    compile_database: &mut CompileDatabase,
    mtime_cache: &mut BTreeMap<String, Option<cu::fs::Time>>
) -> bool {
    // compile commands check (do this first because this is in-memory only check)
    let Some(old_command) = compile_database.get(&command.file) else {
        cu::debug!("need to compile {}: compile database entry does not exist", obj_path.display());
        return false;
    };
    if old_command != &command {
        cu::debug!("need to compile {}: compile command changed", obj_path.display());
        compile_database.update(Cow::Owned(command));
        return false;
    }

    // freshness check
    if !obj_path.exists() {
        cu::debug!("need to compile {}: output does not exist", obj_path.display());
        return false;
    }
    if source_type.uses_depfile() && !dep_path.exists() {
        cu::debug!("need to compile {}: depfile {} does not exist", obj_path.display(), dep_path.display());
        return false;
    }
    let src_mtime = match cu::fs::get_mtime(src_path) {
        Ok(Some(x)) => x,
        Ok(None) => {
            cu::warn!("cannot find source needed to compile {}: {}", obj_path.display(), src_path.display());
            return false;
        }
        Err(e) => {
            cu::warn!("need to compile {}: cannot get mtime of source: {e:?}", obj_path.display());
            return false;
        }
    };
    let out_mtime = match cu::fs::get_mtime(obj_path) {
        Ok(x) => x,
Err(e) => {
        cu::warn!("need to compile {}: cannot get mtime of object: {e:?}", obj_path.display());
        return false;
    }};
    let out_mtime = match out_mtime {
        None => {
            cu::debug!("need to compile {}: source is newer than object", obj_path.display());
            return false;
        }
        Some(t) if t < src_mtime => {
            cu::debug!("need to compile {}: source is newer than object", obj_path.display());
            return false;
        }
        Some(t) => t
    };

    // dependency check
    if !source_type.uses_depfile() {
        cu::debug!("up-to-date: {}: skipped dependency check", obj_path.display());
        return true;
    }
    let depfile= match cu::fs::read_string(dep_path) {
        Err(e) => {
        cu::warn!("need to compile {}: failed to read depfile: {e:?}", obj_path.display());
        return false;
        }
        Ok(x) =>x,
    };
    let depfile = match depfile::parse(&depfile) {
        Err(pos) => {
            cu::warn!(
            "need to compile {}: error when parsing depfile {} at position {pos}",
            obj_path.display(),
                dep_path.display()
            );
            return false;
        }
        Ok(x) => x
    };
    for dep in depfile.recurse_deps(&command.file) {
        let dep_mtime = match mtime_cache.get(dep) {
            None => {
                // this file has not been checked
                let dep_mtime = match cu::fs::get_mtime(dep) {
                    Ok(mtime) => mtime,
                    Err(e) => {
                        // probably file doesn't exist
                        cu::debug!("failed to get mtime when checking dependency: {dep}: {e:?}");
                        None
                    },
                };
                mtime_cache.insert(dep.to_string(), dep_mtime);
                dep_mtime
            }
            Some(t) => *t,
        };
        match dep_mtime {
            None => {
                cu::debug!("need to compile {}: dependency no longer exists: {}", obj_path.display(), dep);
                return false;
            }
            Some(t) if t > out_mtime => {
                cu::debug!("need to compile {}: dependency changed: {}", obj_path.display(), dep);
                return false;
            }
            _ => {}
        }
    }

    cu::debug!("up-to-date: {}: dependencies up-to-date", obj_path.display());
    true
}
