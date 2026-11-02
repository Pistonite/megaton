use std::path::PathBuf;
use std::sync::Arc;

use cu::pre::*;

use megaton_config::cmd::AsProfileFlag;
use megaton_config::config_file::{Config, ConfigLoadOpts, FtpConfig};
use megaton_config::toolchain::ToolchainEnv;
use megaton_packaging as packaging;

use crate::compile_mgr::CompileMgr;
use crate::rust;

/// The `build` subcommand
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    /// The profile for selecting what to build
    #[clap(short = 'p', long)]
    profile: Option<String>,

    /// Emit configuration files only (such as compile_commands.json)
    #[clap(short = 'g', long)]
    configure: bool,

    /// Assume megaton library is already unpacked and do not unpack it again
    #[clap(long)]
    no_lib_unpack: bool,

    /// Do not run checks
    #[clap(long)]
    no_check: bool,

    #[clap(flatten)]
    common: cu::cli::Flags,
}

impl AsProfileFlag for Cmd {
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>> {
        Some(&mut self.profile)
    }
}

impl Cmd {
    pub async fn run(self, dir: Option<&str>) -> cu::Result<()> {
        let mut zombie_tasks = vec![];
        let result = self.run_internal(dir, &mut zombie_tasks).await;
        if !zombie_tasks.is_empty() {
            cu::hint!("waiting for tasks to finish...");
            for task in zombie_tasks {
                let _ = task.co_join_maybe_aborted().await;
            }
        }
        result
    }
    async fn run_internal(self, dir: Option<&str>, zombie_tasks: &mut Vec<cu::co::Handle<cu::Result<()>>>) -> cu::Result<()> {
        let toolchain = Arc::new(ToolchainEnv::resolve()?);
        let config = Arc::new(Config::new_from_project_dir(dir, ConfigLoadOpts { 
            resolve: true, validate: true, cli_profile: self.profile.as_deref(), toolchain: Some(&toolchain) })?);

        if !self.no_lib_unpack {
            // unpack megatonlib
            packaging::unpack_megatonlib_if_needed(&config.project.lib_root)?;
        }

        struct Tasks<'a> {
            zombie_tasks: &'a mut Vec<cu::co::Handle<cu::Result<()>>>,

            /// cxx generated sources to be compiled
            pub cxx_sources: Vec<PathBuf>,
            /// single .o object linked with hermit kernel
            pub object: Option<String>,
            /// the task to produce the .o object above
            pub hermit_link_task: Option<cu::co::Handle<cu::Result<()>>>
        }
        impl Drop for Tasks<'_> {
            fn drop(&mut self) {
                if let Some(t) = self.hermit_link_task.take() {
                    self.zombie_tasks.push(t);
                }
            }
        }
        let mut tasks = Tasks {
            zombie_tasks,
            cxx_sources: vec![],
            object: None,
            hermit_link_task: None
        };


        // build rust if needed
        if config.rust.enabled() {
            let rust_env = cu::check!(toolchain.rust.as_ref(), "rust tools are not installed - please install rust on your system")?;
            if rust_env.megaton.is_none() {
                cu::bail!("megaton rust toolchain is not installed - please run `megaton toolchain install`");
            }
            let rust_project = Arc::new(rust::resolve_project(&config)?);
            cu::debug!("resolved rust project: {rust_project:?}");
            let cxx_version_check_handle = if rust_project.has_cxx {
                if !toolchain.cxxbridge.is_installed() {
                    cu::bail!("cxx dependency is found but cxxbridge-cmd is not installed with megaton - please run `megaton toolchain cxxbridge`")
                }
                // spawn cargo-metadata to check cxx version
                let manifest_path = config.rust.cargo_manifest.clone();
                Some(cu::co::spawn_blocking(move || {
                    rust::check_cxx_version_with_cargo_metadata(&manifest_path)
                }))
            } else {
                None
            };
            if self.configure {
                if rust_project.has_build_script {
                    // run the build script since that may generate rust files that need to run
                    // cxxbridge over
                    rust::run_build_script(&config, &toolchain, rust_env).await?;
                }
            } else {
                // cargo will handle running the build script :)
                rust::run_build_full(&config, &toolchain, rust_env).await?;
            }
            if let Some(handle) = cxx_version_check_handle {
                // ensure cxx version validity
                handle.co_join().await??;
                // generate sources and headers from cxx macros in the rust source
                // this runs even during configure - since those files need
                // to be included in the generated compile_commands.json for language
                // services to work
                tasks.cxx_sources = rust::run_cxxbridge(&config, &toolchain, &rust_project).await?;
            }
            tasks.object = Some(rust_project.linked_path.as_utf8()?.to_string());
            if !self.configure {
                let config = config.clone();
                let toolchain = toolchain.clone();
                let rust_project = rust_project.clone();
                tasks.hermit_link_task = Some(cu::co::spawn(async move {
                    rust::link_hermit_kernel(&config, &toolchain, &rust_project).await
                }));
            }
        }

        let configure_only = self.configure;
        let rust_cxx_sources = std::mem::take(&mut tasks.cxx_sources);
        let result = cu::cli::ctrlc_frame().co_execute(move |ctrlc| async move {
            let mut compile_mgr = CompileMgr::new(configure_only, &config)?;
            let result = drive_compile_mgr(&mut compile_mgr, &config, ctrlc, rust_cxx_sources).await;
            if !configure_only {
                let compile_result = compile_mgr.finish_compiling().await;
                result?;
                compile_result?;
                return Ok(compile_mgr);
            }
            result?;
            cu::Ok(compile_mgr)
        }).await?;

        let Some(mut compile_mgr) = result else {
            cu::bail!("interrupted by ctrl-c");
        };
        if configure_only {
            compile_mgr.save_compile_database()?;
            let _ = compile_mgr.finish_compiling().await;
            cu::info!("compile database saved");
            return Ok(())
        }

        let mut compiled_objects = compile_mgr.into_objects();
        // compile_mgr is dropped, entering linking step


        todo!()
    }
}

/// Compile all the sources
async fn drive_compile_mgr(
    compile_mgr: &mut CompileMgr,
    config: &Config,
    ctrlc: cu::CtrlcSignal,
    rust_cxx_sources: Vec<PathBuf>,
) -> cu::Result<()> {
    let mut error_count = 0;
    for source in rust_cxx_sources {
        if let Err(e) = compile_mgr.compile(&source, ctrlc.clone()) {
            cu::error!("{e:?}");
            error_count += 1;
        }
        if ctrlc.signaled() {
            break;
        }
    }
    if !ctrlc.signaled() {
        for source_dir in &config.build_config()?.sources {
            for entry in cu::fs::walk(source_dir)? {
                if ctrlc.signaled() {
                    break;
                }
                let Ok(entry) = entry else {
                    continue;
                };
                if let Err(e) = compile_mgr.compile(entry.path(), ctrlc.clone()) {
                    cu::error!("{e:?}");
                    error_count += 1;
                }
            }
        }
    }

    if error_count > 0 {
        cu::bail!("{error_count} errors while looking for source files");
    }

    Ok(())
}
