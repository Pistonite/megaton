// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

use std::path::PathBuf;
use std::sync::Arc;

use cu::pre::*;

use megaton_config::cmd::AsProfileFlag;
use megaton_config::config_file::{Config, ConfigLoadOpts};
use megaton_config::toolchain::ToolchainEnv;
use megaton_packaging as packaging;

use crate::checking::Checker;
use crate::compile_mgr::CompileMgr;
use crate::linking;
use crate::npdm;
use crate::nso;
use crate::rust;

/// The `build` subcommand
#[derive(Debug, clap::Parser, AsRef)]
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
    #[as_ref]
    common: cu::cli::Flags,
}

impl AsProfileFlag for Cmd {
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>> {
        Some(&mut self.profile)
    }
}

impl Cmd {
    pub fn run(self, config_path: Option<&str>) -> cu::Result<()> {
        // config/toolchain must be resolved outside of async runtime
        // because they are blocking
        let toolchain = Arc::new(ToolchainEnv::resolve()?);
        let config = Arc::new(Config::new_from_project(
            config_path,
            ConfigLoadOpts {
                resolve: true,
                validate: true,
                cli_profile: self.profile.as_deref(),
                toolchain: Some(&toolchain),
            },
        )?);

        let task = cu::co::spawn(async move {
            let mut zombie_tasks = vec![];
            let result = self
                .run_internal(config, toolchain, &mut zombie_tasks)
                .await;
            if !zombie_tasks.is_empty() {
                cu::debug!("waiting for tasks to finish...");
                for task in zombie_tasks {
                    let _ = task.co_join_maybe_aborted().await;
                }
            }
            result
        });

        task.join().flatten()
    }
    async fn run_internal(
        self,
        config: Arc<Config>,
        toolchain: Arc<ToolchainEnv>,
        zombie_tasks: &mut Vec<cu::co::Handle<cu::Result<()>>>,
    ) -> cu::Result<()> {
        cu::info!(
            "building {} (profile '{}')",
            config.project.module_name,
            config.project.profile
        );

        if !self.no_lib_unpack {
            // unpack megatonlib
            packaging::unpack_megatonlib_if_needed(&config.project.lib_root)?;
        }

        struct Tasks<'a> {
            zombie_tasks: &'a mut Vec<cu::co::Handle<cu::Result<()>>>,

            /// cxx generated sources to be compiled
            pub cxx_sources: Vec<PathBuf>,
            /// single .o object linked with hermit kernel
            pub rust_hermit_object: Option<String>,
            /// the task to produce the .o object above
            pub hermit_link_task: Option<cu::co::Handle<cu::Result<bool>>>,
            /// Task to build libgcc_nounwind.a
            pub libgcc_nounwind_task: Option<cu::co::Handle<cu::Result<()>>>,
            /// Task to load the checker
            pub checker_load_task: Option<cu::co::Handle<cu::Result<Checker>>>,
            pub npdm_task: Option<cu::co::Handle<cu::Result<()>>>,
            pub compile_database_task: Option<cu::co::Handle<cu::Result<()>>>,
        }
        impl Drop for Tasks<'_> {
            fn drop(&mut self) {
                if let Some(t) = self.hermit_link_task.take() {
                    self.zombie_tasks.push(cu::co::spawn(async move {
                        t.co_join().await??;
                        cu::Ok(())
                    }));
                }
                if let Some(t) = self.libgcc_nounwind_task.take() {
                    self.zombie_tasks.push(t);
                }
                if let Some(t) = self.checker_load_task.take() {
                    self.zombie_tasks.push(cu::co::spawn(async move {
                        t.co_join().await??;
                        cu::Ok(())
                    }));
                }
                if let Some(t) = self.npdm_task.take() {
                    self.zombie_tasks.push(t);
                }
                if let Some(t) = self.compile_database_task.take() {
                    self.zombie_tasks.push(t);
                }
            }
        }
        let mut tasks = Tasks {
            zombie_tasks,
            cxx_sources: vec![],
            rust_hermit_object: None,
            hermit_link_task: None,
            libgcc_nounwind_task: None,
            checker_load_task: None,
            npdm_task: None,
            compile_database_task: None,
        };

        // build libgcc_nounwind since it's required for linking later
        if config.build_config()?.std_enabled() {
            let config = config.clone();
            let toolchain = toolchain.clone();
            tasks.libgcc_nounwind_task = Some(cu::co::spawn(async move {
                linking::build_libgcc_nounwind(&config, &toolchain).await
            }));
        }

        // build rust if needed
        if config.rust.enabled() {
            let rust_env = cu::check!(
                toolchain.rust.as_ref(),
                "rust tools are not installed - please install rust on your system"
            )?;
            if rust_env.megaton.is_none() {
                cu::bail!(
                    "megaton rust toolchain is not installed - please run `megaton toolchain install`"
                );
            }
            let rust_project = Arc::new(rust::resolve_project(&config)?);
            cu::debug!("resolved rust project: {rust_project:?}");
            let cxx_version_check_handle = if rust_project.has_cxx {
                if !toolchain.cxxbridge.is_installed() {
                    cu::bail!(
                        "cxx dependency is found but cxxbridge-cmd is not installed with megaton - please run `megaton toolchain cxxbridge`"
                    )
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
            tasks.rust_hermit_object = Some(rust_project.linked_path.as_utf8()?.to_string());
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
        let result = {
            let config = config.clone();
            cu::cli::ctrlc_frame()
                .co_execute(move |ctrlc| async move {
                    let mut compile_mgr = CompileMgr::new(configure_only, &config)?;
                    let result =
                        drive_compile_mgr(&mut compile_mgr, &config, ctrlc, rust_cxx_sources).await;
                    if !configure_only {
                        let compile_result = compile_mgr.finish_compiling().await;
                        result?;
                        compile_result?;
                        return Ok(compile_mgr);
                    }
                    result?;
                    cu::Ok(compile_mgr)
                })
                .await?
        };

        let Some(mut compile_mgr) = result else {
            cu::bail!("interrupted by ctrl-c");
        };
        if configure_only {
            compile_mgr.save_compile_database()?;
            let _ = compile_mgr.finish_compiling().await;
            cu::info!("compile database saved");
            return Ok(());
        }

        let mut relink_needed = compile_mgr.is_any_object_compiled();
        let mut compiled_objects = compile_mgr.take_objects();

        // compile_mgr is moved, entering linking step
        tasks.compile_database_task = Some(cu::co::spawn_blocking(move || {
            compile_mgr.save_compile_database()
        }));

        let bar = cu::progress("linking").spawn();
        compiled_objects.sort();

        let no_check_file_path = config.project.output_root.join(".no_check");

        // preload checker
        if !self.no_check {
            tasks.checker_load_task = {
                let config = config.clone();
                Some(cu::co::spawn(async move { Checker::load(&config).await }))
            };
        } else {
            // write a .no_check file in the output directory.
            // if this file exists, check will force to run even if the output
            // elf didn't change, if --no-check is not specified
            if let Err(e) = cu::fs::write(
                &no_check_file_path,
                "this file exists to ensure check runs the next time",
            ) {
                cu::warn!("failed to write .no_check file: {e:?}");
                cu::warn!(
                    "check might be skipped next time without --no-check if build is successful!"
                );
            };
        }

        if let Some(task) = tasks.hermit_link_task.take() {
            let changed = task.co_join().await??;
            relink_needed |= changed;
        };

        if let Some(task) = tasks.libgcc_nounwind_task.take() {
            task.co_join().await??;
        };

        let link_task = {
            let config = config.clone();
            let toolchain = toolchain.clone();
            let rust_hermit_object = tasks.rust_hermit_object.clone();
            cu::co::spawn(async move {
                let build_config = config.build_config()?;
                linking::link_elf(
                    &config,
                    build_config,
                    &toolchain,
                    relink_needed,
                    &compiled_objects,
                    rust_hermit_object.as_deref(),
                )
                .await
            })
        };

        tasks.npdm_task = {
            let config = config.clone();
            let toolchain = toolchain.clone();
            Some(cu::co::spawn(async move {
                npdm::create_npdm_file(&config, &toolchain).await
            }))
        };

        let did_link = link_task.co_join().await??;
        let mut should_run_elf_to_nso = did_link;

        if did_link {
            bar.done();
        } else {
            bar.done_with_message(&format!(
                "linking: {}.elf is up-to-date",
                config.project.module_name
            ));
        }

        if let Some(task) = tasks.checker_load_task.take() {
            let checker = task.co_join().await??;
            let mut should_check = did_link;
            let has_no_check_file = no_check_file_path.is_file();
            if !should_check && no_check_file_path.is_file() {
                should_check = true;
            }
            should_run_elf_to_nso = checker.check(&config, &toolchain, should_check).await?;
            if has_no_check_file {
                if let Err(e) = cu::fs::remove(no_check_file_path) {
                    cu::warn!("{e:?}");
                }
            }
        }

        if self.no_check {
            cu::hint!("skipped elf checks because --no-check is specified");
        }

        if should_run_elf_to_nso {
            nso::elf_to_nso(&config, &toolchain).await?;
            cu::info!(
                "built {}.nso (profile '{}')",
                config.project.module_name,
                config.project.profile
            );
        } else {
            cu::info!(
                "{}.nso is up-to-date (profile '{}')",
                config.project.module_name,
                config.project.profile
            );
        }

        if let Some(task) = tasks.npdm_task.take() {
            task.co_join().await??;
        }

        if let Some(task) = tasks.compile_database_task.take() {
            if let Err(e) = task.co_join().await.flatten() {
                cu::warn!("failed to save compile database: {e:?}");
            }
        }

        Ok(())
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
