use std::path::{Path, PathBuf};
use std::process::Command;
use cu::pre::*;
use walkdir::WalkDir;

pub struct LibInfo {
    pub crate_name: String,
    pub crate_dir: PathBuf,
    pub src_dir: PathBuf,
}

pub fn lib_info(project_root: &Path) -> cu::Result<LibInfo> {
    let cargo_toml: PathBuf = find_cargo_toml(project_root)?;
    let crate_dir: PathBuf = cargo_toml.parent().context("Cargo.toml has no parent")?.to_path_buf(); 
    let crate_name: String = derive_crate_name(&crate_dir)?;
    let src_dir: PathBuf = crate_dir.join("src");
    if !src_dir.exists() {
        cu::bail!("expected src/ at {}", src_dir.display());
    }
    Ok(LibInfo { crate_name, crate_dir, src_dir })
}

pub fn bridge_files(lib: &LibInfo) -> cu::Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&lib.src_dir) {
        let entry: walkdir::DirEntry = entry?;
        if !entry.file_type().is_file() { continue; }
        let p = entry.path();
        if !is_rust_file(p) { continue; }
        if probe_with_cxxbridge(p)? { out.push(p.to_path_buf()); }
    }
    Ok(out)
}

fn find_cargo_toml(project_root: &Path) -> cu::Result<PathBuf> {
    let rs = project_root.join("rs").join("Cargo.toml");
    if rs.exists() { return Ok(rs); }
    let root = project_root.join("Cargo.toml");
    if root.exists() { return Ok(root); }
    cu::bail!("no Cargo.toml found (tried {} and {})", project_root.join("rs/Cargo.toml").display(), project_root.join("Cargo.toml").display())
}

fn derive_crate_name(crate_dir: &Path) -> cu::Result<String> {
    Ok(crate_dir.file_name().context("crate dir has no name")?.to_string_lossy().to_string())
}

fn is_rust_file(p: &Path) -> bool {
    matches!(p.extension().and_then(|s| s.to_str()), Some("rs"))
}

fn bridge_negative_marker(stderr: &str) -> bool {
    stderr.contains("no cxx::bridge") || stderr.contains("no #[cxx::bridge]") || stderr.contains("no cxx bridge")
}

fn probe_with_cxxbridge(rs: &Path) -> cu::Result<bool> {
    let exe = which::which("cxxbridge").context("cxxbridge not found; `cargo install cxxbridge-cmd`")?;    
    let output = Command::new(&exe)
        .arg("--header")
        .arg(rs)
        .output()
        .with_context(|| format!("spawn {} --header {}", exe.display(), rs.display()))?;
    if output.status.success() {
        return Ok(true);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if bridge_negative_marker(&stderr) {
        Ok(false)
    } else {
        cu::bail!("cxxbridge probe failed for {}\n{}", rs.display(), stderr)
    }
}
