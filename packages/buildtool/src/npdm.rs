// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

use std::path::Path;

use cu::pre::*;

use megaton_config::config_file::Config;
use megaton_config::toolchain::ToolchainEnv;

#[cu::context("failed to create main.npdm")]
pub async fn create_npdm_file(config: &Config, toolchain: &ToolchainEnv) -> cu::Result<()> {
    let npdm_template_file =
        cu::path!(&(&config.project.lib_root) / "mega-cpp" / "etc" / "template.npdm.json");
    let npdm_template_str = cu::fs::read_string(&npdm_template_file)?;
    let mut npdm_data: json::Value = json::parse(&npdm_template_str)?;
    npdm_data["title_id"] = json!(format!("0x{}", config.module.title_id_hex()));

    let out_npdm = &config.project.out_npdm;
    let out_npdm_json = out_npdm.with_added_extension("json");
    cu::fs::write_json_pretty(&out_npdm_json, &npdm_data)?;

    Path::new(&toolchain.devkita64.npdmtool)
        .command()
        .args([&out_npdm_json, out_npdm])
        .all_null()
        .co_wait_nz()
        .await?;

    cu::debug!("created main.npdm");
    Ok(())
}
