use std::path::Path;

use cu::pre::*;

fn main() -> cu::Result<()> {
    let commit_hash = megaton_cli_build::get_commit()?;
    let packages_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent_abs()?;
    let cli_pkg_path = packages_path.join("cli");
    let manifest_path = cli_pkg_path.join("Cargo.toml");

    let info = cu::check!(megaton_cli_build::pack_library(&packages_path, Path::new("temp/megaton-cmd/libmegaton.tar.gz")), "failed to pack library")?;
    let library_hash = info.sha256;

    let isolated_manifest =
    cu::check!(
    megaton_toolchain_build::create_isolated_cargo_manifest_with_deps_removed(&manifest_path, None, [
            "megaton-cli-build"
        ]),
        "failed to create isolated manifest for megaton-cmd"
    )?;

    cu::fs::write("temp/megaton-cmd/Cargo.toml", &isolated_manifest)?;
    cu::fs::write("temp/megaton-cmd/build.rs", format!(r##"
fn main() {{
    println!("cargo::rustc-env=MEGATON_COMMIT={commit_hash}");
    println!("cargo::rustc-env=MEGATON_LIB_SHA256={library_hash}");
}}
"##))?;
    Ok(())
}
