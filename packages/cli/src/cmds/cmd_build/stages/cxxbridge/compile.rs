use std::path::{Path, PathBuf};
use std::process::Command;
use cu::pre::*;
use crate::cmds::cmd_build::config::Flags;

pub fn compile_generated(_flags: &Flags, cc_src: &Path, out_obj: &Path) -> cu::Result<()> {


    if let Some(parent) = out_obj.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }


    let build_dir = {
        let Some(cxx_dir) = cc_src.parent() else {
            cu::bail!("expected build dir above {}", cc_src.display());
        };
        let Some(build_dir) = cxx_dir.parent().and_then(|p| p.parent()) else {
            cu::bail!("failed to locate build dir above {}", cc_src.display());
        };
        build_dir.to_path_buf()
    };


    let include_root: PathBuf = build_dir.join("include");
    let include_rs: PathBuf = include_root.join("rs");  

    let pkg_root = if let Some(p) = build_dir.parent() {
        p
    } else {
        cu::bail!("failed to locate package root");
    };
    let src_cpp: PathBuf = pkg_root.join("rs/src"); 


    let compiler = std::env::var("CXX").unwrap_or_else(|_| "c++".to_string());

    let args: Vec<String> = vec![
        "-std=c++17".into(),
        "-I".into(), include_root.display().to_string(),
        "-I".into(), include_rs.display().to_string(),
        "-I".into(), src_cpp.display().to_string(),
        "-c".into(),
        cc_src.display().to_string(),
        "-o".into(),
        out_obj.display().to_string(),
    ];

    cu::debug!(
        "cxxbridge compile: {} -> {} [{} {}]",
        cc_src.display(),
        out_obj.display(),
        compiler,
        args.join(" ")
    );


    let out = Command::new(&compiler)
        .args(&args)
        .output()
        .with_context(|| format!("failed to spawn {}", compiler))?;


    if !out.status.success() {
        cu::bail!(
            "cxx compile failed (status {}):\nstdout:\n{}\nstderr:\n{}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    Ok(())
}
