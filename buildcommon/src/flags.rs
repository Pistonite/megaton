//! Build flags
//!
//! This is used for both building project source and megaton lib

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{print, Unused};

/// Default flags for `build.flags.common` in Megaton.toml
pub static DEFAULT_COMMON: &[&str] = &[
    "-march=armv8-a+crc+crypto",
    "-mtune=cortex-a57",
    "-mtp=soft",
    "-fPIC",
    "-fvisibility=hidden",
    // debug info
    "-g",
];

/// Default flags for `build.flags.c` in Megaton.toml
///
/// By default, also extends from `DEFAULT_COMMON`
pub static DEFAULT_C: &[&str] = &[
    // strict
    "-Wall",
    "-Werror",
    // size optimization
    "-ffunction-sections",
    "-fdata-sections",
    // needed to make sure functions in headers are inlined
    "-O3",
];

/// Default flags for `build.flags.cxx` in Megaton.toml
///
/// By default, also extends from `DEFAULT_C` (which extends from COMMON)
pub static DEFAULT_CPP: &[&str] = &[
    "-std=c++20",
    // disable c++ features - todo: maybe they can be enabled?
    "-fno-rtti",
    "-fno-exceptions",
    "-fno-asynchronous-unwind-tables",
    "-fno-unwind-tables",
];

/// Default flags for `build.flags.as` in Megaton.toml
///
/// By default, also extends from `DEFAULT_CPP` (which extends from `DEFAULT_C` and
/// `DEFAULT_COMMON`)
pub static DEFAULT_AS: &[&str] = &[];

/// Default flags for `build.flags.ld` in Megaton.toml
///
/// By default, also extends from `DEFAULT_COMMON`
pub static DEFAULT_LD: &[&str] = &[
    "-nostartfiles",
    "-nodefaultlibs",
    "-Wl,--shared",
    "-Wl,--export-dynamic",
    "-Wl,-z,nodynamic-undefined-weak",
    "-Wl,--build-id=sha1",
    // "-Wl,--exclude-libs=ALL",
    // size optimization
    "-Wl,--gc-sections",
    // NX specific
    "-Wl,--nx-module-name",
];

/// Flags resolved from configuration that can be passed to the compiler
#[derive(Debug, Clone, PartialEq)]
pub struct Flags {
    pub cflags: Vec<String>,
    pub cxxflags: Vec<String>,
    pub sflags: Vec<String>,
    pub ldflags: Vec<String>,
}

/// Flags from configuration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FlagConfig {
    pub common: Option<Vec<String>>,
    pub c: Option<Vec<String>>,
    pub cxx: Option<Vec<String>>,
    #[serde(rename = "as")]
    pub as_: Option<Vec<String>>,
    pub ld: Option<Vec<String>>,

    #[serde(flatten, default)]
    pub unused: Unused,
}

macro_rules! create_flags {
    ($field: expr, $default: expr) => {
        match $field {
            None => $default.iter().map(|x| x.to_string()).collect::<Vec<_>>(),
            Some(flags) => {
                let mut v = vec![];
                for flag in flags {
                    if flag == "<default>" {
                        v.extend($default.iter().map(|x| x.to_string()));
                    } else {
                        v.push(flag.clone());
                    }
                }
                v
            }
        }
    };
    ($field: expr, $default: ident extends $base: expr) => {
        match $field {
            None => $base
                .iter()
                .cloned()
                .chain($default.iter().map(|x| x.to_string()))
                .collect::<Vec<_>>(),
            Some(flags) => {
                let mut v = $base.clone();
                for flag in flags {
                    if flag == "<default>" {
                        v.extend($default.iter().map(|x| x.to_string()));
                    } else {
                        v.push(flag.clone());
                    }
                }
                v
            }
        }
    };
}

impl Flags {
    pub fn from_config(config: &FlagConfig) -> Self {
        let common = create_flags!(&config.common, DEFAULT_COMMON);
        let mut cflags = create_flags!(&config.c, DEFAULT_C extends common);

        // no need to check if the flag already exists.. that's O(N)
        // we already said in the docs don't do that
        cflags.push(format!("-fdiagnostics-color={}", print::color_flag()));


        let cxxflags = create_flags!(&config.cxx, DEFAULT_CPP extends cflags);
        let sflags = create_flags!(&config.as_, DEFAULT_AS extends cxxflags);
        let ldflags = create_flags!(&config.ld, DEFAULT_LD extends common);

        Self {
            cflags,
            cxxflags,
            sflags,
            ldflags,
        }
    }

    /// Add define flags (`-D<name>`) for C and C++
    pub fn add_defines(&mut self, defines: impl IntoIterator<Item = impl Display>) {
        let flags = defines
            .into_iter()
            .map(|x| format!("-D{}", x))
            .collect::<Vec<_>>();
        self.cflags.extend(flags.iter().cloned());
        self.cxxflags.extend(flags);
    }

    /// Add include flags (`-I<path>`) for C and C++
    pub fn add_includes(&mut self, includes: impl IntoIterator<Item = impl Display>) {
        let flags = includes
            .into_iter()
            .map(|x| format!("-I{}", x))
            .collect::<Vec<_>>();
        self.cflags.extend(flags.iter().cloned());
        self.cxxflags.extend(flags);
    }

    /// Set `-Wl,-init=<symbol>` for the linker
    #[inline]
    pub fn set_init(&mut self, symbol: impl Display) {
        self.ldflags.push(format!("-Wl,-init={}", symbol));
    }

    /// Set `-Wl,--version-script=<path>` for the linker
    #[inline]
    pub fn set_version_script(&mut self, path: impl Display) {
        self.ldflags.push(format!("-Wl,--version-script={}", path));
    }

    /// Add library paths (`-L<path>`) for the linker
    #[inline]
    pub fn add_libpaths(&mut self, paths: impl IntoIterator<Item = impl Display>) {
        self.ldflags
            .extend(paths.into_iter().map(|x| format!("-L{}", x)));
    }

    /// Add libraries (`-l<name>`) for the linker
    #[inline]
    pub fn add_libraries(&mut self, libs: impl IntoIterator<Item = impl Display>) {
        self.ldflags
            .extend(libs.into_iter().map(|x| format!("-l{}", x)));
    }

    /// Add linker scripts (`-Wl,-T<path>`) for the linker
    #[inline]
    pub fn add_ldscripts(&mut self, scripts: impl IntoIterator<Item = impl Display>) {
        self.ldflags
            .extend(scripts.into_iter().map(|x| format!("-Wl,-T,{}", x)));
    }
}
