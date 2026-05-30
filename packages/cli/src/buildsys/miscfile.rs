
use std::path::Path;

use cu::pre::*;

use crate::env;

pub async fn make_npdm(output_dir: &Path, title_id_hex: &str) -> cu::Result<()> {
    let mut npdm_data: json::Value = json::parse(include_str!("../../template.npdm.json"))?;
    npdm_data["title_id"] = json!(format!("0x{}", title_id_hex));

    let main_npdm_json = output_dir.join("main.npdm.json");
    let main_npdm = output_dir.join("main.npdm");

    cu::fs::co_write_json_pretty(&main_npdm_json, &npdm_data).await?;

    env::get()
        .npdmtool()
        .command()
        .add(cu::args![&main_npdm_json, &main_npdm])
        .all_null()
        .co_wait_nz()
        .await?;

    cu::debug!("created npdm: {}", main_npdm.try_to_rel().display());
    Ok(())
}

pub fn make_verfile(path: &Path, entry: &str) -> cu::Result<()> {
    let verfile_before = "{\n\tglobal:\n\n";
    let verfile_after = ";\n\tlocal: *;\n};";
    let verfile_data = format!("{}{}{}", verfile_before, entry, verfile_after);
    if write_if_changed(path, verfile_data.as_bytes())? {
        cu::debug!("Cmd_build: updated verfile");
    } else {
        cu::debug!("Cmd_build: verfile up to date");
    }
    Ok(())
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> cu::Result<bool> {
    let changed = match cu::fs::read(path) {
        Ok(existing) => existing != bytes,
        Err(_) => true,
    };
    if changed {
        cu::fs::write(path, bytes)?;
    }
    Ok(changed)
}
