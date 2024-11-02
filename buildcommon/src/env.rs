use std::path::PathBuf;

use derive_more::derive::Deref;
use serde::{Deserialize, Serialize};

use error_stack::{report, Result, ResultExt};

use crate::system::{self, PathExt, Error};
use crate::{errorln, hintln, infoln, verboseln};

/// Environment of the tool
#[derive(Debug, Serialize, Deserialize)]
pub struct Env {
    /// Path to root of the megaton repo (MEGATON_HOME)
    #[serde(skip)]
    pub megaton_home: PathBuf,

    /// Path to DevKitPro (DEVKITPRO)
    pub devkitpro: PathBuf,

    /// Path to rustup
    pub rustup: PathBuf,

    /// Path to rustc
    pub rustc: PathBuf,

    /// Path to git
    pub git: PathBuf,

    /// Path to ninja
    pub ninja: PathBuf,

    /// Path to cmake
    pub cmake: PathBuf,
}

impl Env {

    /// Load the environment from cache file
    ///
    /// If no cache exists, it will fallback to [`check`](Self::check)
    /// and create the cache file with [`save`](Self::save)
    pub fn load(home: Option<String>) -> Result<Self, Error> {
        let home = match home {
            Some(home) => PathBuf::from(home),
            None => get_megaton_home()?,
        };

        let cache_path = cache_path_from(&home);
        if cache_path.exists() {
            verboseln!("found cached env: {}", cache_path.display());
            let reader = system::buf_reader(&cache_path)?;
            match serde_yaml_ng::from_reader::<_, Self>(reader) {
                Ok(mut env) => {
                    env.megaton_home = home;
                    return Ok(env);
                }
                Err(e) => {
                    verboseln!("failed to parse cached env: {}", e);
                    verboseln!("falling back to check");
                }
            };
        }

        let env = Self::check_with_home(home, false)?;
        if let Err(e) = env.save() {
            verboseln!("failed to save env: {}", e);
        }

        Ok(env)
    }

    /// Check the environment and tools.
    pub fn check(home: Option<String>) -> Result<Self, Error> {
        let home = match home {
            Some(home) => PathBuf::from(home),
            None => get_megaton_home()?,
        };
        Self::check_with_home(home, true)
    }

    fn check_with_home(home: PathBuf, check_more: bool) -> Result<Self, Error> {
        infoln!("Root", "{}", home.display());
        let mut ok = true;
        let devkitpro = std::env::var("DEVKITPRO").unwrap_or_default();
        let devkitpro = if devkitpro.is_empty() {
            ok = false;
            errorln!("Missing", "DEVKITPRO");
            hintln!("Fix", "Please install DevKitPro");
            hintln!("Fix", "  https://devkitpro.org/wiki/devkitPro_pacman#customising-existing-pacman-install");
            PathBuf::new()
        } else {
            match dunce::canonicalize(&devkitpro) {
                Ok(path) => {
                    if path.display().to_string() != devkitpro {
                        ok = false;
                        errorln!("Invalid", "DEVKITPRO is not absolute");
                        hintln!("Fix", "Please set DEVKITPRO to the absolute path of your DevKitPro installation");
                    } else {
                        infoln!("OK", "Found DEVKITPRO");
                    }
                    path
                }
                Err(_) => {
                    ok = false;
                    errorln!("Missing", "DEVKITPRO");
                    hintln!("Fix", "Set DEVKITPRO to the path of your DevKitPro installation");
                    PathBuf::new()
                }
            }
        };

        let rustup = if let Ok(p) = which::which("rustup") {
            infoln!("OK", "Found rustup");
            p
        } else {
            ok = false;
            errorln!("Missing", "rustup");
            PathBuf::new()
        };

        let rustc = if let Ok(p) = which::which("rustc") {
            infoln!("OK", "Found rustc");
            p
        } else {
            ok = false;
            errorln!("Missing", "rustc");
            PathBuf::new()
        };

        if rustup.as_os_str().is_empty() || rustc.as_os_str().is_empty() {
            hintln!("Fix", "Please install Rust toolchain");
            hintln!("Fix", "  https://rustup.rs/");
        }

        let git = if let Ok(p) = which::which("git") {
            infoln!("OK", "Found git");
            p
        } else {
            ok = false;
            errorln!("Missing", "git");
            hintln!("Fix", "Please install git");
            #[cfg(windows)]
            hintln!("Fix", "  https://git-scm.com/downloads/win");
            PathBuf::new()
        };

        let ninja = if let Ok(p) = which::which("ninja") {
            infoln!("OK", "Found ninja");
            p
        } else {
            ok = false;
            errorln!("Missing", "ninja");
            hintln!("Fix", "Please install ninja-build");

            PathBuf::new()
        };

        let cmake = if let Ok(p) = which::which("cmake") {
            infoln!("OK", "Found cmake");
            p
        } else {
            ok = false;
            errorln!("Missing", "cmake");
            hintln!("Fix", "Please install cmake");

            PathBuf::new()
        };

        let env = Self {
            megaton_home: home,
            devkitpro,
            rustup,
            rustc,
            git,
            ninja,
            cmake,
        };

        let env = if check_more {
            let env_more = EnvMore::from(env);
            env_more.check()?;
            env_more.env
        } else {
            env
        };

        if !ok {
            errorln!("Failed", "Environment check");
            return Err(report!(Error::CheckEnv));
        }

        Ok(env)
    }

    /// Save the environment to cache file
    pub fn save(&self) -> Result<(), Error> {
        let cache_path = self.cache_path();
        let writer = system::buf_writer(&cache_path)?;
        serde_yaml_ng::to_writer(writer, self)
        .change_context_lazy(|| Error::WriteYaml(cache_path.display().to_string()))?;

        infoln!("Cached", "Environment to {}", cache_path.display());
        Ok(())
    }

    /// Get the path to the cache file
    fn cache_path(&self) -> PathBuf {
        cache_path_from(&self.megaton_home)
    }
}

fn cache_path_from(home: &PathBuf) -> PathBuf {
    home.join("bin").into_joined("env_cache.yml")
}



fn get_megaton_home() -> Result<PathBuf, Error> {
    get_megaton_home_internal()
        .change_context(Error::FindToolRoot)
        .attach_printable("Please see README.md for how to setup MEGATON_HOME properly")
}

fn get_megaton_home_internal() -> Result<PathBuf, Error> {
        // should be MEGATON_HOME/target/{release,debug}/megaton-buildtool
        let exe = std::env::current_exe().change_context(Error::CurrentExe)?
            .into_parent()?.into_parent()?.into_parent()?;
        Ok(exe)
}

/// Environment with extra cached paths
#[derive(Debug, Deref)]
pub struct EnvMore {
    #[deref]
    env: Env,

    /// Path to C compiler
    ///
    /// Should be at $DEVKITPRO/devkitA64/bin/aarch64-none-elf-gcc
    pub cc: PathBuf,

    /// Path to C++ compiler
    ///
    /// Should be at $DEVKITPRO/devkitA64/bin/aarch64-none-elf-g++
    pub cxx: PathBuf,

    /// Path to the libnx include directory
    ///
    /// Should be at $DEVKITPRO/libnx/include
    pub libnx_include: PathBuf,

    /// Path to npdmtool
    ///
    /// Should be at $DEVKITPRO/tools/bin/npdmtool
    pub npdmtool: PathBuf,

    /// Path to objdump
    ///
    /// Should be at $DEVKITPRO/devkitA64/bin/aarch64-none-elf-objdump
    pub objdump: PathBuf,

    /// Path to elf2nso
    ///
    /// Should be at $DEVKITPRO/tools/bin/elf2nso
    pub elf2nso: PathBuf,
}

impl From<Env> for EnvMore {
    fn from(env: Env) -> Self {
        let devkitpro = &env.devkitpro;
        let libnx_include = devkitpro.join("libnx").into_joined("include");

        let devkita64_bin = devkitpro.join("devkitA64").into_joined("bin");
        let cc = devkita64_bin.join("aarch64-none-elf-gcc");
        let cxx = devkita64_bin.join("aarch64-none-elf-g++");
        let objdump = devkita64_bin.into_joined("aarch64-none-elf-objdump");


        let tools_bin = devkitpro.join("tools").into_joined("bin");
        let npdmtool = tools_bin.join("npdmtool");
        let elf2nso = tools_bin.into_joined("elf2nso");

        Self {
            env,
            cc,
            cxx,
            libnx_include,
            npdmtool,
            objdump,
            elf2nso,
        }
    }
}

impl EnvMore {
    pub fn check(&self) -> Result<(), Error> {
        let mut ok = true;
        if self.libnx_include.exists() {
            infoln!("OK", "Found libnx/include");
        } else {
            ok = false;
            errorln!("Missing", "libnx/include");
            hintln!("Fix", "(Re-)install DevKitPro");
        }

        match which::which(&self.cc) {
            Ok(p) if p == self.cc => infoln!("OK", "Found aarch64-none-elf-gcc"),
            _ => {
                ok = false;
                errorln!("Missing", "aarch64-none-elf-gcc");
                hintln!("Fix", "(Re-)install DevKitPro");
            }
        }

        match which::which(&self.cxx) {
            Ok(p) if p == self.cxx => infoln!("OK", "Found aarch64-none-elf-g++"),
            _ => {
                ok = false;
                errorln!("Missing", "aarch64-none-elf-g++");
                hintln!("Fix", "(Re-)install DevKitPro");
            }
        }

        match which::which(&self.objdump) {
            Ok(p) if p == self.objdump => infoln!("OK", "Found aarch64-none-elf-objdump"),
            _ => {
                ok = false;
                errorln!("Missing", "aarch64-none-elf-objdump");
                hintln!("Fix", "(Re-)install DevKitPro");
            }
        }

        match which::which(&self.elf2nso) {
            Ok(p) if p == self.elf2nso => infoln!("OK", "Found elf2nso"),
            _ => {
                ok = false;
                errorln!("Missing", "elf2nso");
                hintln!("Fix", "(Re-)install DevKitPro");
            }
        }

        match which::which(&self.npdmtool) {
            Ok(p) if p == self.npdmtool => infoln!("OK", "Found npdmtool"),
            _ => {
                ok = false;
                errorln!("Missing", "npdmtool");
                hintln!("Fix", "(Re-)install DevKitPro");
            }
        }

        if !ok {
            errorln!("Failed", "Environment check");
            return Err(report!(Error::CheckEnv));
        }

        Ok(())
    }
}
