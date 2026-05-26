use std::path::Path;

use cu::pre::*;

fn main() -> cu::Result<()> {
    let mut toolchain_pkg_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent_abs()?;
    toolchain_pkg_path.push("toolchain-build");
    let toolchain_manifest_path = toolchain_pkg_path.join("Cargo.toml");

    let isolated_manifest =
    cu::check!(
    megaton_toolchain_build::create_isolated_cargo_manifest(&toolchain_manifest_path, None),
        "failed to create isolated manifest for megaton-toolchain-build"
    )?;
    cu::fs::write("temp/toolchain-build/Cargo.toml", &isolated_manifest)?;
    Ok(())
}
