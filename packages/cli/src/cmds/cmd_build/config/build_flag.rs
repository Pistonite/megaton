// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::fmt::Display;

use cu::pre::*;

use super::{CaptureUnused, ExtendProfile, Validate, ValidateCtx};

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
    pub rust: Option<Vec<String>>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
}

impl Validate for FlagConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}

impl ExtendProfile for FlagConfig {
    fn extend_profile(&mut self, other: &Self) {
        extend_flags(&mut self.common, &other.common);
        extend_flags(&mut self.c, &other.c);
        extend_flags(&mut self.cxx, &other.cxx);
        extend_flags(&mut self.as_, &other.as_);
        extend_flags(&mut self.ld, &other.ld);
    }
}

fn extend_flags(dst: &mut Option<Vec<String>>, src: &Option<Vec<String>>) {
    match (dst.as_mut(), src) {
        (_, None) => {}
        (None, Some(flags)) => {
            // dst none = ["<default>"]
            let mut new_flags = flags.clone();
            if !new_flags.iter().any(|x| x == "<default>") {
                new_flags.push("<default>".to_string());
            }
            *dst = Some(new_flags);
        }
        (Some(dst_flags), Some(src_flags)) => {
            for flag in src_flags {
                if !dst_flags.contains(flag) {
                    dst_flags.push(flag.clone());
                }
            }
        }
    }
}

/// Flags resolved from configuration that can be passed to the compiler
#[derive(Debug, Clone, PartialEq)]
pub struct Flags {
    pub cflags: Vec<String>,
    pub cxxflags: Vec<String>,
    pub sflags: Vec<String>,
    pub ldflags: Vec<String>,
    pub rustflags: String,
}

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
    // size optimization
    "-Wl,--gc-sections",
    // NX specific
    "-Wl,--nx-module-name",
];

pub static DEFAULT_RUST: &[&str] = &[];

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
        let color_flag = if cu::color_enabled() {
            "never"
        } else {
            "always"
        };
        cflags.push(format!("-fdiagnostics-color={color_flag}"));

        let cxxflags = create_flags!(&config.cxx, DEFAULT_CPP extends cflags);
        let sflags = create_flags!(&config.as_, DEFAULT_AS extends cxxflags);
        let ldflags = create_flags!(&config.ld, DEFAULT_LD extends common);

        let rustflags = create_flags!(&config.rust, DEFAULT_RUST).join(" ");

        Self {
            cflags,
            cxxflags,
            sflags,
            ldflags,
            rustflags,
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
        self.ldflags.push(format!("-Wl,-init={symbol}"));
    }

    /// Set `-Wl,--version-script=<path>` for the linker
    #[inline]
    pub fn set_version_script(&mut self, path: impl Display) {
        self.ldflags.push(format!("-Wl,--version-script={path}"));
    }

    /// Add library paths (`-L<path>`) for the linker
    #[inline]
    pub fn add_libpaths(&mut self, paths: impl IntoIterator<Item = impl Display>) {
        self.ldflags
            .extend(paths.into_iter().map(|x| format!("-L{x}")));
    }

    /// Add libraries (`-l<name>`) for the linker
    #[inline]
    pub fn add_libraries(&mut self, libs: impl IntoIterator<Item = impl Display>) {
        self.ldflags
            .extend(libs.into_iter().map(|x| format!("-l{x}")));
    }

    /// Add linker scripts (`-Wl,-T<path>`) for the linker
    #[inline]
    pub fn add_ldscripts(&mut self, scripts: impl IntoIterator<Item = impl Display>) {
        self.ldflags
            .extend(scripts.into_iter().map(|x| format!("-Wl,-T,{x}")));
    }
}
