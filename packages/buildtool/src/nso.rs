use std::path::Path;

use cu::pre::*;

use megaton_config::config_file::Config;
use megaton_config::toolchain::ToolchainEnv;

/// Convert ELF to NSO
#[cu::context("failed to convert elf to nso")]
pub async fn elf_to_nso(config: &Config, toolchain: &ToolchainEnv) -> cu::Result<()> {
    let elf_path = &config.project.out_elf;
    let nso_path = &config.project.out_nso;
    let elf2nso = &toolchain.devkita64.elf2nso;

    Path::new(elf2nso)
        .command()
        .args([elf_path, nso_path])
        .stdin_null()
        .stdout(cu::lv::D)
        .stderr(cu::lv::E)
        .co_wait_nz()
        .await?;

    cu::debug!("nso created");
    Ok(())
}
