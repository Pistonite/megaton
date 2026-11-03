use std::path::{Path, PathBuf};

use cu::pre::*;

use megaton_config::toolchain::ToolchainEnv;
use megaton_config::config_file::Config;

use crate::hashing;

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
        cu::debug!("libgcc.a.sha256 is up-to-date and output exists, skipping building libgcc_nounwind.a");
        return Ok(());
    }

    let temp_output_path = config.project.out_lib.join("libgcc_nounwind.a.temp");
    cu::fs::copy(&libgcc_original_path, &temp_output_path)?;

    let ar = &toolchain.devkita64.ar;
    let (child,stdout,stderr) = Path::new(ar).command()
        .add(cu::args!["d", &temp_output_path, "unwind-dw2.o", "unwind-dw2-fde.o"])
        .stdout(cu::pio::buffer())
        .stderr(cu::pio::buffer())
        .stdin_null()
        .co_spawn().await?;
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
    cu::debug!("built {}", output_path.display());
    Ok(())
}

pub async fn link_elf(
    config: &Config,
    force_relink: bool,
    source_objects: &[String],
    rust_hermit_object: Option<&str>,


) -> cu::Result<()> {
    let link_command_path = config.project.output_root.join("link_command.json");
    let mut is_link_needed = force_relink;

    // construct dependency list and library list
    let mut dependencies = vec![];
    let mut arguments = vec![];
    
}

#[derive(Debug, Serialize, Deserialize)]
struct LinkCommand {
    /// The dependent object files to check for timestamp
    dependencies: Vec<String>,
    /// The arguments including the path to the driver binary
    arguments: Vec<String>,
}

fn get_libgcc_nounwind_path(config: &Config) -> PathBuf {
    config.project.out_lib.join("libgcc_nounwind.a")
}
