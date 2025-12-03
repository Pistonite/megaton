// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use cu::{Result, Context, Command, pio, which};
use cu::pre::*;
use super::RustCrate;
use std::path::{PathBuf, Path};


pub fn generate_cxx_bridge_src(rust_crate: RustCrate, module_target_path: &Path) -> Result<()> {
    // TODO: Parse rust crate for cxxbridge files 
    //
    // TODO: Place generated headers in {module}/include/rust/
    //
    // TODO: Place generated source in {module}/src/cxxbridge



    let crate_src = rust_crate.path.join("test_mod/src");
    if !crate_src.exists() {
        cu::debug!("generate_cxx_bridge_src: no src/ at {}", crate_src.display());
        return Ok(());
    }

    let include_rust = module_target_path.join("include").join("rust");
    let cxx_src_dir  = module_target_path.join("src").join("cxxbridge");
    
    
    cu::fs::make_dir(&include_rust)?;
    cu::fs::make_dir(&cxx_src_dir)?;

    let bridge_files = find_bridge_files(&crate_src)?;
    if bridge_files.is_empty() {
        cu::debug!("cxxbridge: no #[cxx::bridge] files found under {}", crate_src.display());
        return Ok(());
    }

    let exe = which("cxxbridge").context("cxxbridge not found; `cargo install cxxbridge-cmd`")?;

    for rs in bridge_files {

        let stem_os = rs.file_stem()
            .ok_or_else(|| cu::fmterr!("invalid file name: {}", rs.display()))?;

        let stem = stem_os.to_string_lossy();



        let out_h  = include_rust.join(format!("{stem}.h"));
        let out_cc = cxx_src_dir.join(format!("{stem}.cc"));

        if let Some(p) = out_h.parent()  { cu::fs::make_dir(p)?; }
        if let Some(p) = out_cc.parent() { cu::fs::make_dir(p)?; }

        cu::debug!("cxxbridge: spawning {} -> {}, {}", rs.display(), out_h.display(), out_cc.display());

        let ( header_child,  h_stdout,  h_stderr) = Command::new(&exe)
            .arg("--header")
            .arg(&rs)
            .stdout(pio::buffer())
            .stderr(pio::buffer())
            .stdin_null()
            .spawn()
            .with_context(|| format!("spawn {} --header {}", exe.display(), rs.display()))?;

        let status = header_child.wait()?;
        let h_stdout_bytes: Vec<u8> = h_stdout.join()??;
        let h_stderr_bytes: Vec<u8> = h_stderr.join()??;

        if !status.success() {
            cu::bail!(
                "cxxbridge --header failed for {}: {}",
                rs.display(),
                String::from_utf8_lossy(&h_stderr_bytes).trim(),
            );
        }

        write_if_changed(&out_h, &h_stdout_bytes)?;


        let ( cc_child, c_stdout, c_stderr) = Command::new(&exe)
            .arg(&rs)
            .stdout(pio::buffer())
            .stderr(pio::buffer())
            .stdin_null()
            .spawn()
            .with_context(|| format!("spawn {} {}", exe.display(), rs.display()))?;

        let status = cc_child.wait()?;
        let c_stdout_bytes: Vec<u8> = c_stdout.join()??;
        let c_stderr_bytes: Vec<u8> = c_stderr.join()??;


        if !status.success() {
            cu::bail!(
                "cxxbridge (cc) failed for {}: {}",
                rs.display(),
                String::from_utf8_lossy(&c_stderr_bytes).trim(),
            );
        }
        write_if_changed(&out_cc, &c_stdout_bytes)?;

    }
    Ok(())
}



fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    if let Some(parent) = path.parent() {
        cu::fs::make_dir(parent)?;
    }

    let changed = match cu::fs::read(path) {
        Ok(existing) => existing != bytes,
        Err(_) => true,
    };

    if changed {
        let tmp = path.with_extension("tmp");
        cu::fs::write(&tmp, bytes)?;
        cu::fs::copy(&tmp, path)?;
        cu::fs::remove(&tmp)?;
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
    let mut it = cu::fs::walk(dir)?;
    while let Some(entry) = it.next() {
        let entry = entry?;
        let p = entry.path();

        if p.extension().is_some_and(|e| e == "rs") {
            out.push(p.to_path_buf());
        }
    }
    Ok(())
}

fn probe_has_cxxbridge(rs: &Path) -> Result<bool> {
    let exe = which("cxxbridge")
        .context("cxxbridge not found; install with `cargo install cxxbridge-cmd`")?;

    let ( cmd,  _,  cmd_stderr) = Command::new(&exe)
        .arg("--header")
        .arg(rs)
        .stdout(pio::buffer())
        .stderr(pio::buffer())
        .stdin_null()
        .spawn()
        .with_context(|| format!("probe spawn {}", rs.display()))?;


    let status = cmd.wait()?;
    let cmd_stderr_bytes: Vec<u8> = cmd_stderr.join()??;

    if status.success() {
        return Ok(true);
    }

    let stderr = String::from_utf8_lossy(&cmd_stderr_bytes);

    if stderr.contains("no cxx::bridge")
        || stderr.contains("no #[cxx::bridge]")
        || stderr.contains("no cxx bridge")
    {
        return Ok(false);
    }

    cu::bail!("cxxbridge probe failed for {}:\n{}", rs.display(), stderr)
}