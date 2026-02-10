// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::cmds::cmd_build::BTArtifacts;

use super::RustCrate;

pub fn generate_cxx_bridge_src(
    rust_crate: &RustCrate,
    bt_artifacts: &BTArtifacts,
) -> cu::Result<()> {
    let include_rust = &bt_artifacts.module_include;
    let cxx_src_dir = &bt_artifacts.module_src;

    // cu::fs::make_dir(include_rust)?;
    // cu::fs::make_dir(cxx_src_dir)?;

    let bridge_files = find_bridge_files(rust_crate)?;
    if bridge_files.is_empty() {
        cu::debug!("cxxbridge: no #[cxx::bridge] files found",);
        return Ok(());
    }

    let exe =
        cu::which("cxxbridge").context("cxxbridge not found; `cargo install cxxbridge-cmd`")?;

    let (cxx_child, cxx_stdout, cxx_stderr) = cu::Command::new(&exe)
        .arg("--header")
        .stdout(cu::pio::buffer())
        .stderr(cu::pio::buffer())
        .stdin_null()
        .spawn()
        .with_context(|| format!("spawn {} --header", exe.display()))?;

    let status = cxx_child.wait()?;
    let cxx_stdout_bytes = cxx_stdout.join()??;
    let cxx_stderr_bytes = cxx_stderr.join()??;

    if !status.success() {
        cu::bail!(
            "cxxbridge --header failed failed: {}",
            String::from_utf8_lossy(&cxx_stderr_bytes).trim()
        );
    }

    write_if_changed(&include_rust.join("rust").join("cxx.h"), &cxx_stdout_bytes)?;

    for rs in bridge_files {
        let stem_os = rs
            .file_stem()
            .ok_or_else(|| cu::fmterr!("invalid file name: {}", rs.display()))?;

        let stem = stem_os.to_string_lossy();
        let suffix = rust_crate.header_suffix.clone();
        let path_to_rs = rs.canonicalize().unwrap();

        // let out_h = include_rust.join(format!("{stem}.{suffix}"));
        // let out_cc = cxx_src_dir.join(format!("{stem}.cc"));
        let rel_source_path = rust_crate
            .source_paths
            .iter()
            .find(|p| {
                let v = path_to_rs.starts_with(p);
                cu::info!("pref {:?} rs {:?} v {:?}", p, path_to_rs, v);
                v
            })
            .unwrap();
        let rel_source_path = path_to_rs.strip_prefix(rel_source_path)?;
        let mut out_h = bt_artifacts.module_include.join(rel_source_path);
        let mut out_cc = bt_artifacts.module_src.join(rel_source_path);
        out_h.set_file_name(format!("{stem}{suffix}"));
        out_cc.set_file_name(format!("{stem}.cc"));
        if let Some(p) = out_h.parent() {
            cu::fs::make_dir(p)?;
        }
        if let Some(p) = out_cc.parent() {
            cu::fs::make_dir(p)?;
        }

        cu::debug!(
            "cxxbridge: spawning {} -> {}, {}",
            rs.display(),
            out_h.display(),
            out_cc.display()
        );

        let (header_child, h_stdout, h_stderr) = cu::Command::new(&exe)
            .arg("--header")
            .arg(&rs)
            .stdout(cu::pio::buffer())
            .stderr(cu::pio::buffer())
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

        let (cc_child, c_stdout, c_stderr) = cu::Command::new(&exe)
            .arg(&rs)
            .stdout(cu::pio::buffer())
            .stderr(cu::pio::buffer())
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

fn write_if_changed(path: &Path, bytes: &[u8]) -> cu::Result<bool> {
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

fn find_bridge_files(rust_crate: &RustCrate) -> cu::Result<Vec<PathBuf>> {
    let source_files = rust_crate.get_source_files()?;

    let mut cxxbridge_rs_files = Vec::new();
    for p in source_files {
        if probe_has_cxxbridge(&p)? {
            cxxbridge_rs_files.push(p);
        }
    }
    Ok(cxxbridge_rs_files)
}

fn probe_has_cxxbridge(rs: &Path) -> cu::Result<bool> {
    let exe = cu::which("cxxbridge")
        .context("cxxbridge not found; install with `cargo install cxxbridge-cmd`")?;

    let (cmd, _, cmd_stderr) = cu::Command::new(&exe)
        .arg("--header")
        .arg(rs)
        .stdout(cu::pio::buffer())
        .stderr(cu::pio::buffer())
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
