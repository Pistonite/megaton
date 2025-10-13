use std::path::PathBuf;
use cu::pre::*;
use super::super::config::Flags;

mod discover;
mod generate;
mod paths;
mod compile;

use crate::cmds::cmd_build::config::Config;
use crate::cmds::cmd_build::CmdBuild;

pub fn run(config: &Config, _args: &CmdBuild, flags: &mut Flags) -> cu::Result<Vec<PathBuf>> {
    if !config.megaton.rust_enabled {
        cu::debug!("cxxbridge: skipped (megaton.rust_enabled = false)");
        return Ok(Vec::new());
    }

    let project_root = std::env::current_dir().context("determine project root")?;
    let lib = discover::lib_info(&project_root).context("discover Rust [lib] crate")?;
    let bridge_files = discover::bridge_files(&lib).context("scan/probe Rust sources")?;
    if bridge_files.is_empty() {
        cu::debug!("cxxbridge: no #[cxx::bridge] files found");
        return Ok(Vec::new());
    }

    let include_root = paths::include_root(&project_root);
    flags.add_includes([format!("-I{}", include_root.display())]);

    let mut produced = Vec::with_capacity(bridge_files.len());
    for rs in bridge_files {
        let (out_h, out_cc, out_o) = paths::map_outputs(&project_root, &lib, &rs)
            .with_context(|| format!("map outputs for {}", rs.display()))?;
        cu::debug!("cxxbridge: {} -> {}, {}", rs.display(), out_h.display(), out_cc.display());

        let generated = generate::run_cxxbridge(&rs)
            .with_context(|| format!("cxxbridge codegen for {}", rs.display()))?;
        generate::write_if_changed(&out_h, &generated.header)
            .with_context(|| format!("write header {}", out_h.display()))?;
        generate::write_if_changed(&out_cc, &generated.cc)
            .with_context(|| format!("write source {}", out_cc.display()))?;
        compile::compile_generated(flags, &out_cc, &out_o)
            .with_context(|| format!("compile {}", out_cc.display()))?;
        produced.push(out_o);
    }

    Ok(produced)
}
