// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use cu::pre::*;

static ENVIRONMENT: OnceLock<Environment> = OnceLock::new();

pub fn commit() -> &'static str {
    env!("MEGATON_COMMIT")
}

pub fn get() -> &'static Environment {
    ENVIRONMENT.get().expect("environment was not initialized")
}

/// Initialize the environment
pub fn init() -> cu::Result<()> {
    let megaton_home = megaton_toolchain_build::get_megaton_home()?;

    let devkitpro = cu::check!(cu::env_var("DEVKITPRO"), "DEVKITPRO environment variable not set; please ensure devkitpro is installed.")?;
    let devkitpro = Path::new(&devkitpro).normalize()?;

    let env = cu::check!(Environment::new(megaton_home, devkitpro), "failed to initialize environment")?;
    env.debug();
    if ENVIRONMENT.set(env).is_err() {
        cu::bail!("unexpected: environment was already set before init_env()");
    }

    Ok(())
}

// Core environment variables needed to run the tool
// Includes paths to build/debug utilities and caches
#[derive(Debug)]
pub struct Environment {
    megaton_home: PathBuf,

    cc: PathBuf,  // C compiler
    cxx: PathBuf, // C++ compiler
    asm: PathBuf, // Assembler
    ar: PathBuf,  // Archiver
    objdump: PathBuf,
    npdmtool: PathBuf,
    elf2nso: PathBuf,

    cc_version: String,

    devkitpro: PathBuf,
    dkp_includes: Vec<String>,

    cxxbridge: Option<PathBuf>,
}

impl Environment {
    fn new(megaton_home: PathBuf, devkitpro: PathBuf) -> cu::Result<Self> {
        let devkita64 = devkitpro.join("devkitA64");
        let dkp_bin =   devkita64.join("bin");
        let cc = dkp_bin.join("aarch64-none-elf-gcc");
        let cxx = dkp_bin.join("aarch64-none-elf-g++");
        let asm = dkp_bin.join("aarch64-none-elf-gcc");
        let ar = dkp_bin.join("aarch64-none-elf-ar");
        let objdump = dkp_bin.join("aarch64-none-elf-objdump");

        // FIXME: some of these checks requires running the bins
        // which might be able to get parallelized while parsing the config :)

        let dkp_tools_bin = devkitpro.join("tools").join("bin");
        let npdmtool = dkp_tools_bin.join("npdmtool");
        let elf2nso = dkp_tools_bin.join("elf2nso");

        let cc_version = get_cc_version(&cc)?;

        // FIXME: remove if not needed
        // let dkp_version = get_dkp_version(&devkita64, &cc)
        //     .expect("Failed to init environment: check that DKP is installed correctly");
        let dkp_includes = get_dkp_includes(&devkita64, &cc_version)?;

        let cxxbridge = megaton_toolchain_build::cxxbridge::binary_path(&megaton_home).ok();
        Ok(Self {
            megaton_home,
            cc,
            cxx,
            asm,
            ar,
            objdump,
            npdmtool,
            elf2nso,
            cc_version,
            devkitpro,
            dkp_includes,
            cxxbridge,
        })
    }

    /// Get the home of the megaton cache directory
    pub fn home(&self) -> &Path {
        &self.megaton_home
    }
    pub fn dkp_path(&self) -> &Path {
        &self.devkitpro
    }
    pub fn dkp_includes(&self) -> &[String] {
        &self.dkp_includes
    }
    pub fn cc(&self) -> &Path {
        &self.cc
    }
    pub fn cxx(&self) -> &Path {
        &self.cxx
    }
    pub fn asm(&self) -> &Path {
        &self.asm
    }
    pub fn ar(&self) -> &Path {
        &self.ar
    }
    pub fn objdump(&self) -> &Path {
        &self.objdump
    }
    pub fn npdmtool(&self) -> &Path {
        &self.npdmtool
    }
    pub fn elf2nso(&self) -> &Path {
        &self.elf2nso
    }
    pub fn cc_version(&self) -> &str {
        &self.cc_version
    }
    pub fn cxxbridge(&self) -> cu::Result<&Path> {
        cu::check!(self.cxxbridge.as_deref(), "cxxbridge not found; please run `megaton toolchain install`")
    }

    /// Print the environment for debugging
    pub fn debug(&self) {
        cu::debug!("MEGATON_HOME={}", self.megaton_home.display());
        cu::debug!("cc: {}", self.cc.display());
        cu::debug!("cxx: {}", self.cxx.display());
        cu::debug!("as: {}", self.asm.display());
        cu::debug!("ar: {}", self.ar.display());
        cu::debug!("objdump: {}", self.objdump.display());
        cu::debug!("npdmtool: {}", self.npdmtool.display());
        cu::debug!("elf2nso: {}", self.elf2nso.display());
        cu::debug!("compiler version: {}", self.cc_version);
        cu::debug!("system header paths: {:#?}", self.dkp_includes);
        match &self.cxxbridge {
            None => {
                cu::debug!("cxxbridge: megaton toolchain not installed");
            }
            Some(x) => {
                cu::debug!("cxxbridge: {}", x.display());
            }
        }
    }
}

// FIXME: remove if not needed
// fn get_dkp_version(devkita64: &Path, cc: &Path) -> cu::Result<Option<String>> {
//     // /opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/
//     let cpp_path = {
//         let mut p = devkita64.join("aarch64-none-elf");
//         p.extend(["include", "c++"]);
//         p
//     };
//     let readdir = cu::check!(cu::fs::read_dir(&cpp_path), "failed to read c++ include paths from devkitpro")?;
//     let mut candidate = None;
//     for entry in readdir {
//         let entry = match entry {
//             Err(e) => {
//                 cu::warn!("error reading some entry from devkitpro c++ include path: {e:?}");
//                 continue;
//             }
//             Ok(x) => x
//         };
//         let version = match entry.file_name().into_utf8() {
//             Err(e) => {
//                 cu::warn!("ignoring non-utf8 while reading devkitpro c++ version: {e:?}");
//                 continue;
//             }
//             Ok(x) => x
//         };
//         match candidate {
//             None => {
//                 candidate = Some(version);
//             }
//             Some(x) => {
//             }
//         }
//
//
//     }
//
//     // let dir = readdir.filter_map(|x| x.ok()).collect::<Vec<_>>();
//     //
//     // if dir.len() == 1 {
//     //     Ok(dir[0].file_name().display().to_string())
//     // } else {
//     //     // Fallback: query gcc for version
//     //     get_cc_version(cc)
//     // }
// }

#[cu::context("failed to get devkitpro compiler version (path: '{}')", cc_path.display())]
fn get_cc_version(cc_path: &Path) -> cu::Result<String> {
    let (child, _, output) = cc_path.command()
        .arg("-v")
        .stdio_null()
        .stderr(cu::pio::string())
        .spawn()?;
    child.wait_nz()?;
    let output = output.join()??;
    let verline = output.lines().last().unwrap_or(&output);
    let Some(verstring) = verline.split(" ").nth(2) else {
        cu::error!("cannot determine version from cc output:\n{output}");
        cu::bail!("cannot determine cc version: failed to parse output");
    };

    Ok(verstring.to_owned())
}

#[cu::context("failed to get devkitpro system include paths")]
fn get_dkp_includes(devkita64: &Path, cc_version: &str) -> cu::Result<Vec<String>> {
    // /opt/devkitpro/devkitA64/aarch64-none-elf/include
    let arch_include_path = {
        let mut p = devkita64.join("aarch64-none-elf");
        p.push("include");
        p
    };
    // /opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/<ver>
    let cpp_include_path = {
        let mut p = arch_include_path.join("c++");
        p.push(cc_version);
        p
    };
    // /opt/devkitpro/devkitA64/lib/gcc/aarch64-none-elf/<ver>
    let gcc_include_path = {
        let mut p = devkita64.join("lib");
        p.extend(["gcc", "aarch64-none-elf", cc_version]);
        p
    };

    Ok(vec![
        arch_include_path.into_utf8()?,
        cpp_include_path.join("aarch64-none-elf").into_utf8()?,
        cpp_include_path.join("backward").into_utf8()?,
        cpp_include_path.into_utf8()?,
        gcc_include_path.join("include").into_utf8()?,
        {
            let mut p = gcc_include_path;
            p.push("include-fixed");
            p.into_utf8()?
        }
    ])
}




