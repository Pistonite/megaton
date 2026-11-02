// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::ffi::OsStr;
use std::path::Path;

use megaton_config::config_file::{BuildConfig, Config};
use megaton_config::toolchain::ToolchainEnv;

/// Source language type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceType {
    /// C Source
    C,
    /// C++ Source
    Cpp,
    /// Assembly Source without preprocessing
    Asm,
    /// Assembly Source with preprocessing
    AsmPre,
}

impl SourceType {
    pub fn from_extension(ext: &OsStr) -> Option<Self> {
        match ext.as_encoded_bytes() {
            [b'c' | b'C'] => Some(Self::C),
            [b'c' | b'C', b'p' | b'P', b'p' | b'P']
            | [b'c' | b'C', b'+', b'+']
            | [b'c' | b'C', b'x' | b'X', b'x' | b'X']
            | [b'c' | b'C', b'c' | b'C'] => Some(Self::Cpp),
            [b's'] | [b'a' | b'A', b's' | b'S', b'm' | b'M'] => Some(Self::Asm),
            [b'S'] => Some(Self::AsmPre),
            _ => None,
        }
    }

    #[inline]
    pub fn get_compiler(self, config: &BuildConfig) -> &str {
        match self {
            SourceType::C => config.compiler.cc(),
            SourceType::Cpp => config.compiler.cxx(),
            SourceType::Asm => config.compiler.cc(),
            SourceType::AsmPre => config.compiler.cc(),
        }
    }

    #[inline]
    pub fn get_flags(self, config: &BuildConfig) -> &Vec<String> {
        match self {
            SourceType::C => &config.flags.resolved.c,
            SourceType::Cpp => &config.flags.resolved.cxx,
            SourceType::Asm => &config.flags.resolved.cc_asm,
            SourceType::AsmPre => &config.flags.resolved.cc_asm
        }
    }
    #[inline]
    pub fn get_clangd_compat_flags(self, config: &BuildConfig) -> &Vec<String> {
        match self {
            SourceType::C => &config.flags.resolved.compdb.c,
            SourceType::Cpp => &config.flags.resolved.compdb.cxx,
            SourceType::Asm => &config.flags.resolved.compdb.cc_asm,
            SourceType::AsmPre => &config.flags.resolved.compdb.cc_asm
        }
    }

    #[inline]
    pub fn uses_depfile(self) -> bool {
        matches!(self, Self::C | Self::Cpp | Self::AsmPre)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ext(s: &str) -> Option<SourceType> {
        SourceType::from_extension(OsStr::new(s))
    }

    #[test]
    fn c_extensions() {
        assert_eq!(ext("c"), Some(SourceType::C));
        assert_eq!(ext("C"), Some(SourceType::C));
    }

    #[test]
    fn cpp_extensions() {
        assert_eq!(ext("c++"), Some(SourceType::Cpp));
        assert_eq!(ext("C++"), Some(SourceType::Cpp));
        assert_eq!(ext("cxx"), Some(SourceType::Cpp));
        assert_eq!(ext("cpp"), Some(SourceType::Cpp));
        assert_eq!(ext("CpP"), Some(SourceType::Cpp));
        assert_eq!(ext("CXX"), Some(SourceType::Cpp));
        assert_eq!(ext("cXx"), Some(SourceType::Cpp));
        assert_eq!(ext("cc"), Some(SourceType::Cpp));
        assert_eq!(ext("CC"), Some(SourceType::Cpp));
    }

    #[test]
    fn assembly_extensions() {
        assert_eq!(ext("s"), Some(SourceType::Asm));
        assert_eq!(ext("S"), Some(SourceType::AsmPre));
        assert_eq!(ext("asm"), Some(SourceType::Asm));
        assert_eq!(ext("ASM"), Some(SourceType::Asm));
        assert_eq!(ext("Asm"), Some(SourceType::Asm));
    }

    #[test]
    fn unknown_extensions() {
        assert_eq!(ext("rs"), None);
        assert_eq!(ext("h"), None);
        assert_eq!(ext("hpp"), None);
        assert_eq!(ext("txt"), None);
        assert_eq!(ext(""), None);
    }
}
