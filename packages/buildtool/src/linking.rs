use std::path::{Path, PathBuf};

use cu::pre::*;

use megaton_config::config_file::{BuildConfig, Config};
use megaton_config::toolchain::ToolchainEnv;

use crate::hashing;
use crate::printing;

/// Build our libgcc_nounwind.a, which is libgcc with the unwind symbols excluded
/// (since we use the nnsdk ones)
#[cu::context("failed to build libgcc_nounwind.a")]
pub async fn build_libgcc_nounwind(config: &Config, toolchain: &ToolchainEnv) -> cu::Result<()> {
    let libgcc_original_path = &toolchain.devkita64.libs.gcc;
    let libgcc_original_sha256 = hashing::co_file_sha256(libgcc_original_path.as_ref()).await?;

    let sha256_path = config.project.out_lib.join("libgcc.a.sha256");
    let output_path = get_libgcc_nounwind_path(config);
    let mut up_to_date = false;

    match cu::fs::read_string(&sha256_path) {
        Err(e) => {
            cu::debug!("failed to read libgcc.a.sha256, assuming building is needed: {e:?}");
        }
        Ok(current_hash) => {
            if current_hash.trim() != libgcc_original_sha256.trim() {
                cu::debug!("libgcc.a.sha256 is not up-to-date");
            } else {
                up_to_date = true;
            }
        }
    }

    if up_to_date && output_path.is_file() {
        cu::debug!(
            "libgcc.a.sha256 is up-to-date and output exists, skipping building libgcc_nounwind.a"
        );
        return Ok(());
    }

    let temp_output_path = config.project.out_lib.join("libgcc_nounwind.a.temp");
    cu::fs::copy(libgcc_original_path, &temp_output_path)?;

    let ar = &toolchain.devkita64.ar;
    let (child, stdout, stderr) = Path::new(ar)
        .command()
        .add(cu::args![
            "d",
            &temp_output_path,
            "unwind-dw2.o",
            "unwind-dw2-fde.o"
        ])
        .stdout(cu::pio::buffer())
        .stderr(cu::pio::buffer())
        .stdin_null()
        .co_spawn()
        .await?;
    let result = child.co_wait_nz().await;
    if let Err(e) = result {
        if let Ok(stderr) = stderr.co_join().await.flatten() {
            cu::print!("{}", String::from_utf8_lossy(&stderr));
        }
        if let Ok(stdout) = stdout.co_join().await.flatten() {
            cu::print!("{}", String::from_utf8_lossy(&stdout));
        }
        return Err(e);
    }

    cu::fs::copy(&temp_output_path, &output_path)?;
    if let Err(e) = cu::fs::write(sha256_path, libgcc_original_sha256) {
        cu::warn!("{e:?}");
    };
    cu::debug!("built {}", output_path.display());
    Ok(())
}

/// Link the ELF to a temporary location. Return true if not already up-to-date
#[cu::context("linking failed")]
pub async fn link_elf(
    config: &Config,
    build_config: &BuildConfig,
    toolchain: &ToolchainEnv,
    force_relink: bool,
    source_objects: &[String],
    rust_hermit_object: Option<&str>,
) -> cu::Result<bool> {
    let mut is_link_needed = force_relink;

    // construct dependency list and library list
    let mut dependencies = vec![];
    let mut arguments = vec![];

    // use the g++ driver for linking
    let cxx_ld = build_config.compiler.cxx_ld();

    // compiler and link flags
    arguments.push(cxx_ld.to_string());
    arguments.extend(build_config.flags.resolved.cxx_ld.iter().cloned());

    // source objects
    arguments.extend_from_slice(source_objects);
    dependencies.extend_from_slice(source_objects);
    if let Some(o) = rust_hermit_object {
        arguments.push(o.to_string());
        dependencies.push(o.to_string());
    }

    // crt
    let lib = cu::path!(&(&config.project.lib_root) / "mega-cpp" / "lib");
    for o in ["rocrt.o", "rocrt0.o"] {
        let crt_o = cu::check!(
            lib.join(o).into_utf8(),
            "library object paths must be utf-8; please move the project to another location"
        )?;
        arguments.push(crt_o.clone());
        dependencies.push(crt_o);
    }

    // extra objects
    for o in &build_config.objects {
        let o_str = cu::check!(
            o.as_utf8(),
            "extra object paths must be utf-8; please move the project to another location"
        )?;
        arguments.push(o_str.to_string());
        dependencies.push(o_str.to_string());
    }

    // libraries
    if build_config.std_enabled() {
        // lib megaton first - dependent on C/C++ libs
        let libmegacpp_a = cu::check!(
            lib.join("libmegacpp.a").into_utf8(),
            "library path must be utf-8; please move the project to another location"
        )?;
        arguments.push(libmegacpp_a.clone());
        dependencies.push(libmegacpp_a);
        // C++ libs
        let libs = &toolchain.devkita64.libs;

        arguments.push(libs.stdcppexp.clone());
        dependencies.push(libs.stdcppexp.clone());
        arguments.push(libs.stdcpp.clone());
        dependencies.push(libs.stdcpp.clone());
        arguments.push(libs.m.clone());
        dependencies.push(libs.m.clone());

        arguments.push("-Wl,--start-group".to_string());

        arguments.push(libs.sysbase.clone());
        dependencies.push(libs.sysbase.clone());
        arguments.push(libs.c.clone());
        dependencies.push(libs.c.clone());

        let gcc_nounwind = get_libgcc_nounwind_path(config).into_utf8()?;
        arguments.push(gcc_nounwind.clone());
        dependencies.push(gcc_nounwind);

        arguments.push("-Wl,--end-group".to_string());
    }

    arguments.push("-o".to_string());
    arguments.push(config.project.out_elf.clone().into_utf8()?);

    let link_command_path = config.project.output_root.join("link_command.json");
    let link_command = LinkCommand { arguments };
    if let Err(e) = cu::fs::write_json_pretty(&link_command_path, &link_command) {
        cu::warn!("failed to write link_command.json: {e:?}");
    }

    if !is_link_needed {
        is_link_needed = !is_elf_up_to_date(
            config,
            &link_command_path,
            &link_command.arguments,
            &dependencies,
        );
        if !is_link_needed {
            return Ok(false);
        }
    }

    let (child, stdout, stderr) = Path::new(cxx_ld)
        .command()
        .args(&link_command.arguments[1..])
        .stdout(cu::pio::buffer())
        .stderr(cu::pio::buffer())
        .stdin_null()
        .co_spawn()
        .await?;
    let result = child.co_wait_nz().await;
    if let Err(e) = result {
        if let Some(stderr) = printing::join_stdstream(stderr).await {
            cu::error!("{stderr}");
        }
        if let Some(stdout) = printing::join_stdstream(stdout).await {
            cu::print!("{stdout}");
        }
        return Err(e);
    }

    cu::debug!("linking successful");
    Ok(true)
}

fn is_elf_up_to_date(
    config: &Config,
    link_command_path: &Path,
    arguments: &[String],
    dependencies: &[String],
) -> bool {
    let elf_mtime = match cu::fs::get_mtime(&config.project.out_elf) {
        Err(_) | Ok(None) => {
            // output probably doesn't exist
            cu::debug!("need to relink: output elf is missing or not accessible");
            return false;
        }
        Ok(Some(x)) => x,
    };

    let Ok(command) = cu::fs::read_string(link_command_path) else {
        cu::debug!("need to relink: link_command.json is not readable");
        return false;
    };
    let Ok(command) = json::parse::<LinkCommand>(&command) else {
        cu::debug!("need to relink: link_command.json is not valid json");
        return false;
    };
    if command.arguments != arguments {
        cu::debug!("need to relink: link arguments changed");
        return false;
    }
    for dep in dependencies {
        let mtime = match cu::fs::get_mtime(dep) {
            Err(e) => {
                cu::warn!("error access link dependency: {e:?}");
                return false;
            }
            Ok(None) => {
                cu::warn!("link dependency not found: {dep}");
                return false;
            }
            Ok(Some(mtime)) => mtime,
        };
        if mtime > elf_mtime {
            cu::debug!("need to relink: dependency changed: {dep}");
            return false;
        }
    }
    cu::debug!("skipped relinking: dependencies up-to-date");
    true
}

#[derive(Debug, Serialize, Deserialize)]
struct LinkCommand {
    /// The arguments including the path to the driver binary
    arguments: Vec<String>,
}

fn get_libgcc_nounwind_path(config: &Config) -> PathBuf {
    config.project.out_lib.join("libgcc_nounwind.a")
}
