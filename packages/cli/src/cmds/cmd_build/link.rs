// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

// This modules handles compiling c/c++/asm/rust code
use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::env::environment;

/// Link a list of artifacts into an elf file.
pub async fn build_elf(
    need_link: bool,
    artifacts: Vec<PathBuf>,
    ldflags: Vec<String>,
    out_path: &Path,
    link_cmd_path: &Path,
) -> cu::Result<bool> {
    let env = environment();

    let mut args = ldflags;
    for artifact in artifacts {
        args.push(artifact.into_utf8()?);
    }

    args.push(format!("-o{}", out_path.to_owned().into_utf8()?));

    let linker = env.cc_path();
    let link_cmd = LinkCmd::new(linker, &args);
    let old_link_cmd = LinkCmd::try_load(link_cmd_path).ok();

    if let Some(old_link_cmd) = old_link_cmd {
        cu::debug!("linkcmd successfully loaded");
        if !need_link {
            cu::debug!("link not needed");
            if link_cmd.args == old_link_cmd.args {
                cu::debug!("link command unchanged");
                return Ok(false);
            }
            if link_cmd.args != old_link_cmd.args {
                cu::debug!("link command different");
            }
        }
    }

    cu::debug!("linking: {}", link_cmd.display(),);
    link_cmd.execute().await?;
    link_cmd.save(link_cmd_path)?;

    Ok(true)
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
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
            // TODO: fix progress indicator (elf2nso also)
            .stdout(cu::pio::spinner("Linking to ELF").info())
            .stderr(cu::lv::E);
        let child = command.co_spawn().await?.0;
        child.co_wait_nz().await?;

        Ok(())
    }

    fn display(&self) -> String {
        format!("{} {}", self.linker.display(), self.args.join(" "))
    }
}

pub async fn build_nso(elf_path: &Path, nso_path: &Path) -> cu::Result<()> {
    let elf2nso = environment().elf2nso_path();
    cu::debug!(
        "converting elf to nso: {} {} {}",
        elf2nso.display(),
        elf_path.display(),
        nso_path.display()
    );
    let command = elf2nso
        .command()
        .args([elf_path, nso_path])
        .stdin_null()
        .stdout(cu::pio::spinner("Building NSO").info())
        .stderr(cu::lv::E);
    let child = command.co_spawn().await?.0;

    child.co_wait_nz().await
}

// #[cfg(test)]
// mod tests {
//
// }
