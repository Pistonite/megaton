use crate::prelude::*;

use std::path::{Path, PathBuf};

use derive_more::derive::Deref;
use serde::{Deserialize, Serialize};

use crate::system::Error;

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

    /// Path to cargo
    pub cargo: PathBuf,

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
    pub fn load(home: Option<&str>) -> Result<Self, Error> {
        let env = match home {
            Some(home) => {
                if let Some(env) = Self::load_from_cache(home)? {
                    return Ok(env);
                }
                Self::check_with_home(home, false)?
            }
            None => {
                let home = get_megaton_home()?;
                if let Some(env) = Self::load_from_cache(&home)? {
                    return Ok(env);
                }
                Self::check_with_home(home, false)?
            }
        };

        match env {
            Some(env) => {
                env.save();
                Ok(env)
            }
            None => {
                errorln!("Failed", "Cannot initialize environment");
                hintln!("Consider", "Fix the issues above and try again");
                Err(report!(Error::InitEnv))
            }
        }
    }

    #[inline]
    fn load_from_cache(home: impl AsRef<Path>) -> Result<Option<Self>, Error> {
        let home = home.as_ref();
        let cache_path = cache_path_from(home);
        if cache_path.exists() {
            verboseln!("found cached env: {}", cache_path.display());
            let reader = system::buf_reader(&cache_path)?;
            match serde_yaml_ng::from_reader::<_, Self>(reader) {
                Ok(mut env) => {
                    env.megaton_home = home.to_path_buf();
                    return Ok(Some(env));
                }
                Err(e) => {
                    verboseln!("failed to parse cached env: {}", e);
                    verboseln!("falling back to check");
                }
            };
        }

        Ok(None)
    }

    /// Check the environment and tools.
    ///
    /// If check fails, returns Ok(None)
    pub fn check(home: Option<&str>) -> Result<Option<Self>, Error> {
        match home {
            Some(home) => Self::check_with_home(home, true),
            None => Self::check_with_home(get_megaton_home()?, true),
        }
    }

    fn check_with_home(home: impl AsRef<Path>, check_more: bool) -> Result<Option<Self>, Error> {
        let home = home.as_ref();
        infoln!("Root", "{}", home.display());
        let mut ok = true;
        let devkitpro = std::env::var("DEVKITPRO").unwrap_or_default();
        let devkitpro = if devkitpro.is_empty() {
            ok = false;
            errorln!("Missing", "DEVKITPRO");
            hintln!("Fix", "Please install DevKitPro");
            hintln!(
                "Fix",
                "  https://devkitpro.org/wiki/devkitPro_pacman#customising-existing-pacman-install"
            );
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
                    hintln!(
                        "Fix",
                        "Set DEVKITPRO to the path of your DevKitPro installation"
                    );
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

        let cargo = if let Ok(p) = which::which("cargo") {
            infoln!("OK", "Found cargo");
            p
        } else {
            ok = false;
            errorln!("Missing", "cargo");
            PathBuf::new()
        };

        if rustup.as_os_str().is_empty()
            || rustc.as_os_str().is_empty()
            || cargo.as_os_str().is_empty()
        {
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
            megaton_home: home.to_path_buf(),
            devkitpro,
            rustup,
            rustc,
            cargo,
            git,
            ninja,
            cmake,
        };

        let env = if check_more {
            let env_more = RootEnv::from(env);
            ok &= env_more.check()?;
            env_more.env
        } else {
            env
        };

        if !ok {
            return Ok(None);
        }

        Ok(Some(env))
    }

    /// Save the environment to cache file
    pub fn save(&self) {
        match self.save_internal() {
            Ok(_) => {
                infoln!("Cached", "Environment");
            }
            Err(e) => {
                hintln!("Failed", "Environment not cached");
                verboseln!("error: {}", e);
            }
        }
    }

    fn save_internal(&self) -> Result<(), Error> {
        let cache_path = self.cache_path();
        let writer = system::buf_writer(&cache_path)?;
        serde_yaml_ng::to_writer(writer, self)
            .change_context_lazy(|| Error::WriteFile(cache_path.display().to_string()))?;

        Ok(())
    }

    /// Get the path to the cache file
    fn cache_path(&self) -> PathBuf {
        cache_path_from(&self.megaton_home)
    }
}

fn cache_path_from(home: &Path) -> PathBuf {
    home.join("bin").into_joined("env_cache.yml")
}

error_context!(FindToolRoot, |r| -> Error {
    errorln!("Failed", "Megaton repo not found");
    hintln!("Problem", "The tool might be incorrectly installed");
    hintln!("Consider", "See README.md for more info");
    r.change_context(Error::FindToolRoot)
        .attach_printable("Please see README.md for how to install the tool properly")
});
fn get_megaton_home() -> ResultIn<PathBuf, FindToolRoot> {
    // should be MEGATON_HOME/target/{release,debug}/EXE
    let exe = std::env::current_exe()?
        .into_parent()?
        .into_parent()?
        .into_parent()?;
    Ok(exe)
}

/// Environment with extra cached paths
#[derive(Debug, Deref)]
pub struct RootEnv {
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

    /// Path to the libnx lib directory
    ///
    /// Should be at $DEVKITPRO/libnx/lib
    pub libnx_lib: PathBuf,

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

impl From<Env> for RootEnv {
    fn from(env: Env) -> Self {
        let devkitpro = &env.devkitpro;
        let libnx = devkitpro.join("libnx");

        let libnx_include = libnx.join("include");
        let libnx_lib = libnx.into_joined("lib");

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
            libnx_lib,
            npdmtool,
            objdump,
            elf2nso,
        }
    }
}

macro_rules! check_dkp_tool {
    ($ok:ident, $path:expr, $tool:literal) => {
        match which::which($path) {
            Ok(p) if &p == $path => infoln!("OK", concat!("Found ", $tool)),
            _ => {
                $ok = false;
                errorln!("Missing", $tool);
                hintln!("Fix", "(Re-)install DevKitPro");
            }
        }
    };
}

impl RootEnv {
    /// Check root env. Return false if check fails
    pub fn check(&self) -> Result<bool, Error> {
        let mut ok = true;
        if self.libnx_include.exists() {
            infoln!("OK", "Found libnx/include");
        } else {
            ok = false;
            errorln!("Missing", "libnx/include");
            hintln!("Fix", "(Re-)install DevKitPro");
        }

        let libnx = self.libnx_lib.join("libnx.a");
        if libnx.exists() {
            infoln!("OK", "Found libnx.a");
        } else {
            ok = false;
            errorln!("Missing", "libnx.a");
            hintln!("Fix", "(Re-)install DevKitPro");
        }

        check_dkp_tool!(ok, &self.cc, "aarch64-none-elf-gcc");
        check_dkp_tool!(ok, &self.cxx, "aarch64-none-elf-g++");
        check_dkp_tool!(ok, &self.objdump, "aarch64-none-elf-objdump");

        let ar = self.get_dkp_bin("aarch64-none-elf-ar");
        check_dkp_tool!(ok, &ar, "aarch64-none-elf-ar");

        let as_ = self.get_dkp_bin("aarch64-none-elf-as");
        check_dkp_tool!(ok, &as_, "aarch64-none-elf-as");

        check_dkp_tool!(ok, &self.elf2nso, "elf2nso");
        check_dkp_tool!(ok, &self.npdmtool, "npdmtool");

        Ok(ok)
    }

    /// Get path to additional tool in DevKitPro
    ///
    /// Tools not used during a project build is not automatically initialized
    /// and cached to help with build performance
    pub fn get_dkp_bin(&self, tool: &str) -> PathBuf {
        self.devkitpro.join("devkitA64").into_joined("bin").into_joined(tool)
    }
}

/// Environment of a project
#[derive(Debug, Deref)]
pub struct ProjectEnv {
    #[deref]
    env: RootEnv,

    /// Path to the root of the project (where Megaton.toml is)
    pub root: PathBuf,

    /// The target directory for megaton (<root>/target/megaton/<profile>)
    pub target: PathBuf,

    /// The object file output directory (<root>/target/megaton/<profile>/o)
    pub target_o: PathBuf,

    /// The version script file for linker (<root>/target/megaton/<profile>/verfile)
    pub verfile: PathBuf,

    /// The compile_commands.json file (<root>/target/megaton/<profile>/compile_commands.json)
    pub cc_json: PathBuf,

    /// The target ELF (<root>/target/megaton/<profile>/<name>.elf)
    pub elf: PathBuf,

    /// The target NSO (<root>/target/megaton/<profile>/<name>.nso)
    pub nso: PathBuf,
}

impl ProjectEnv {
    /// Load the environment of a project
    ///
    /// `home` is the path to root of metagon repo, and `root`
    /// is root of the project
    pub fn load(
        home: Option<&str>,
        root: PathBuf,
        profile: &str,
        module: &str,
    ) -> Result<Self, Error> {
        let env = Env::load(home)?.into();

        let target = root
            .join("target")
            .into_joined("megaton")
            .into_joined(profile);
        let target_o = target.join("o");
        let verfile = target.join("verfile");
        let cc_json = target.join("compile_commands.json");
        let elf = target.join(format!("{}.elf", module));
        let nso = target.join(format!("{}.nso", module));

        let env = Self {
            env,
            root,
            target,
            target_o,
            verfile,
            cc_json,
            elf,
            nso,
        };

        Ok(env)
    }

    /// Get the path as relative from project root
    pub fn from_root(&self, path: impl AsRef<Path>) -> PathBuf {
        path.as_ref().rebase(&self.root)
    }
}

error_context!(pub FindProjectRoot, |r| -> Error {
    errorln!("Failed", "Project root not found");
    hintln!("Consider", "Run inside a Megaton project or use the -C switch");
    r.change_context(Error::FindProjectRoot)
        .attach_printable("Please run inside a Megaton project")
});

/// Find the directory that contains Megaton.toml
///
/// Prints error message when not found
pub fn find_root(dir: &str) -> ResultIn<PathBuf, FindProjectRoot> {
    let cwd = Path::new(dir).to_abs()?;
    let mut root: &Path = cwd.as_path();
    while !root.join("Megaton.toml").exists() {
        // for some reason borrow analysis is not working here
        // root = root.parent_or_err()?;
        root = root.parent().ok_or(Error::ParentPath)?;
    }
    Ok(root.to_path_buf())
}
