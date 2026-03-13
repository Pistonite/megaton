// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

// This modules handles compiling c/c++/asm/rust code
use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::env::environment;

/// Link a list of artifacts into an elf file.
pub async fn build_elf(
    need_link: bool,
    mut objects: Vec<PathBuf>,
    static_libs: Vec<PathBuf>,
    ldflags: Vec<String>,
    out_path: &Path,
    link_cmd_path: &Path,
) -> cu::Result<bool> {
    let env = environment();
    let mut args = ldflags;

    // sort so args are always comparable regardless of compilation order
    objects.sort();
    for object in objects {
        args.push(object.into_utf8()?);
    }
    for lib in static_libs {
        args.push(lib.into_utf8()?);
    }

    args.push(format!("-o{}", out_path.to_owned().into_utf8()?));

    let linker = env.cc();
    let link_cmd = LinkCmd::new(linker, &args);
    let old_link_cmd = LinkCmd::try_load(link_cmd_path).ok();

    if let Some(old_link_cmd) = old_link_cmd {
        cu::debug!("Link: loaded linkcmd {}", link_cmd_path.display());
        if !need_link && link_cmd == old_link_cmd && out_path.exists() {
            cu::debug!("Link: elf up to date {}", out_path.display());
            return Ok(false);
        }
    }

    link_cmd.execute().await?;
    link_cmd.save(link_cmd_path)?;
    cu::debug!("Link: built elf {}", out_path.display());

    Ok(true)
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
struct LinkCmd {
    pub linker: PathBuf,
    pub args: Vec<String>,
}

impl LinkCmd {
    fn try_load(path: &Path) -> cu::Result<Self> {
        json::read::<LinkCmd>(cu::fs::read(path)?.as_slice())
    }

    fn new(ld_path: &Path, args: &[String]) -> Self {
        Self {
            linker: ld_path.to_path_buf(),
            args: args.to_vec(),
        }
    }

    fn save(&self, path: &Path) -> cu::Result<()> {
        let file = std::fs::File::create(path)?;
        json::write_pretty(file, self)
    }

    async fn execute(&self) -> cu::Result<()> {
        let command = self
            .linker
            .command()
            .args(&self.args)
            .stdin_null()
            .stdout(cu::pio::spinner("Linking").debug())
            .stderr(cu::lv::E);
        let (child, spinner) = command.co_spawn().await?;
        child.co_wait_nz().await?;
        spinner.done();
        Ok(())
    }
}

pub async fn build_nso(elf_path: &Path, nso_path: &Path) -> cu::Result<()> {
    let elf2nso = environment().elf2nso();
    let command = elf2nso
        .command()
        .args([elf_path, nso_path])
        .stdin_null()
        .stdout(cu::pio::spinner("Converting to NSO").debug())
        .stderr(cu::lv::E);
    let (child, spinner) = command.co_spawn().await?;
    let res = child.co_wait_nz().await;
    spinner.done();
    cu::debug!("Link: converted to nso {}", nso_path.display());
    res
}

// #[cfg(test)]
// mod tests {
//
// }
