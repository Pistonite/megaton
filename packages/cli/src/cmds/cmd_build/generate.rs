// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::path::Path;
use cu::{Result, Context, Command, Spawn, lv, Child};
use super::RustCrate;
use std::fs;
use std::path::PathBuf;


pub fn generate_cxx_bridge_src(rust_crate: RustCrate, module_target_path: impl AsRef<Path>) -> Result<()> {
    // TODO: Parse rust crate for cxxbridge files 
    //
    // TODO: Place generated headers in {module}/include/rust/
    //
    // TODO: Place generated source in {module}/src/cxxbridge



    let crate_src = rust_crate.path.join("src");
    if !crate_src.exists() {
        cu::debug!("generate_cxx_bridge_src: no src/ at {}", crate_src.display());
        return Ok(());
    }

    let module_root = module_target_path.as_ref();
    let include_rust = module_root.join("include").join("rust");
    let cxx_src_dir  = module_root.join("src").join("cxxbridge");
    fs::create_dir_all(&include_rust)
        .with_context(|| format!("mkdir -p {}", include_rust.display()))?;
    fs::create_dir_all(&cxx_src_dir)
        .with_context(|| format!("mkdir -p {}", cxx_src_dir.display()))?;

    let bridge_files = find_bridge_files(&crate_src)?;
    if bridge_files.is_empty() {
        cu::debug!("cxxbridge: no #[cxx::bridge] files found under {}", crate_src.display());
        return Ok(());
    }

    let exe = std::env::var_os("CXXBRIDGE").unwrap_or_else(|| "cxxbridge".into());
    let mut children: Vec::<(PathBuf, PathBuf, Child)> = Vec::new();

    for rs in bridge_files {
        let stem = rs.file_name().unwrap().to_string_lossy();
        let out_h  = include_rust.join(format!("{stem}.h"));
        let out_cc = cxx_src_dir.join(format!("{stem}.cc"));
        let tmp_h  = out_h.with_extension("h.tmp");
        let tmp_cc = out_cc.with_extension("cc.tmp"); 

        cu::debug!("cxxbridge: spawning {} -> {}, {}", rs.display(), out_h.display(), out_cc.display());

        let header_child = Command::new(&exe)
            .arg("--header")
            .arg(&rs)
            .arg("--output")
            .arg(&tmp_h)
            .stdout(lv::I)
            .stderr(lv::E)
            .stdin_null()
            .spawn()
            .with_context(|| format!("{:?} --header {}", exe, rs.display()))?;

        let cc_child = Command::new(&exe)
            .arg(&rs)
            .arg("--output")
            .arg(&tmp_cc)
            .stdout(lv::I)
            .stderr(lv::E)
            .stdin_null()
            .spawn()
            .with_context(|| format!("{:?} {}", exe, rs.display()))?;

        children.push((out_h, tmp_h, header_child));
        children.push((out_cc, tmp_cc, cc_child));
    }

    for (out_path, tmp_path, child) in children {
        let status = child.wait()
            .with_context(|| format!("wait for cxxbridge {}", out_path.display()))?;
        if !status.success() {
            cu::bail!("cxxbridge failed for {}", out_path.display());
        }

        let bytes = std::fs::read(&tmp_path)
            .with_context(|| format!("read tmp output {}", tmp_path.display()))?;
        write_if_changed(&out_path, &bytes)?;
        let _ = std::fs::remove_file(&tmp_path);
    }

    Ok(())
}



fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir -p {}", parent.display()))?;
    }
    let changed = match fs::read(path) { Ok(existing) => existing != bytes, Err(_) => true };
    if changed {
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, bytes).with_context(|| format!("write tmp {}", tmp.display()))?;
        fs::rename(&tmp, path).with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    }
    Ok(changed)
}

fn find_bridge_files(src_root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_rs(src_root, &mut out)?;
    let mut keep = Vec::new();
    for p in out {
        if probe_has_cxxbridge(&p)? {
            keep.push(p);
        }
    }
    Ok(keep)
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_rs(&p, out)?;
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(p);
        }
    }
    Ok(())
}

fn probe_has_cxxbridge(rs: &Path) -> Result<bool> {
    use std::process::Command as StdCmd;

    let exe = which::which("cxxbridge")
        .context("cxxbridge not found; `cargo install cxxbridge-cmd`")?;
    let out = StdCmd::new(&exe)
        .arg("--header")
        .arg(rs)
        .output()
        .with_context(|| format!("spawn {} --header {}", exe.display(), rs.display()))?;

    if out.status.success() {
        return Ok(true);
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("no cxx::bridge") || stderr.contains("no #[cxx::bridge]") || stderr.contains("no cxx bridge") {
        Ok(false)
    } else {
        cu::bail!("cxxbridge probe failed for {}\n{}", rs.display(), stderr)
    }
}

