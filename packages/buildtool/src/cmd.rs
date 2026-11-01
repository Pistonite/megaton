use cu::pre::*;

use megaton_config::cmd::AsProfileFlag;
use megaton_config::config_file::{Config, ConfigLoadOpts, FtpConfig};
use megaton_config::toolchain::ToolchainEnv;
use megaton_packaging as packaging;

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
        let toolchain = ToolchainEnv::resolve()?;
        let config = Config::new_from_project_dir(dir, ConfigLoadOpts { 
            resolve: true, validate: true, cli_profile: self.profile.as_deref(), toolchain: Some(&toolchain) })?;

        if !self.no_lib_unpack {
            // unpack megatonlib
            packaging::unpack_megatonlib_if_needed(&config.project.lib_root)?;
        }

        // build rust if needed
        if config.rust.enabled() {
            let rust_env = cu::check!(toolchain.rust.as_ref(), "rust tools are not installed - please install rust on your system")?;
            if rust_env.megaton.is_none() {
                cu::bail!("megaton rust toolchain is not installed - please run `megaton toolchain install`");
            }
            let rust_project = rust::resolve_project(&config)?;
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
                rust::run_cxxbridge(&config, &toolchain, &rust_project).await?;

                // TODO: create compile manager, add sources to it
                // and let it compile
            }


        }

        todo!()
    }
}
