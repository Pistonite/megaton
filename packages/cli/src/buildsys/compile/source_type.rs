// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::ffi::OsStr;
use std::path::Path;

use crate::config::Flags;
use crate::env::Environment;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceType {
    C,
    Cpp,
    Assembly,
}

impl SourceType {
    pub fn from_extension(ext: &OsStr) -> Option<Self> {
        match ext.as_encoded_bytes() {
            [ b'c' | b'C' ] => Some(Self::C),
            [ b'c' | b'C', b'p' | b'P', b'p' | b'P' ]|
            [ b'c' | b'C', b'+', b'+' ]|
            [ b'c' | b'C', b'x' | b'X', b'x' | b'X' ]|
            [ b'c' | b'C', b'c' | b'C']
            => Some(Self::Cpp),
            [ b's' | b'S' ]|
            [ b'a'| b'A', b's' | b'S', b'm'| b'M' ]
            => Some(Self::Assembly),
            _ => None
        }
    }

    #[inline]
    pub fn get_compiler<'a>(self, env: &'a Environment) -> &'a Path {
        match self {
            SourceType::C => env.cc(),
            SourceType::Cpp => env.cxx(),
            SourceType::Assembly => env.cc(),
        }
    }

    #[inline]
    pub fn get_flags<'a>(self, flags: &'a Flags) -> &'a Vec<String> {
        match self {
            SourceType::C => &flags.cflags,
            SourceType::Cpp => &flags.cxxflags,
            SourceType::Assembly => &flags.sflags,
        }
    }

    #[inline]
    pub fn uses_depfile(self) -> bool {
        matches!(self, Self::C | Self::Cpp)
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
        assert_eq!(ext("cc"),  Some(SourceType::Cpp));
        assert_eq!(ext("CC"),  Some(SourceType::Cpp));
    }

    #[test]
    fn assembly_extensions() {
        assert_eq!(ext("s"),   Some(SourceType::Assembly));
        assert_eq!(ext("S"),   Some(SourceType::Assembly));
        assert_eq!(ext("asm"), Some(SourceType::Assembly));
        assert_eq!(ext("ASM"), Some(SourceType::Assembly));
        assert_eq!(ext("Asm"), Some(SourceType::Assembly));
    }

    #[test]
    fn unknown_extensions() {
        assert_eq!(ext("rs"),  None);
        assert_eq!(ext("h"),   None);
        assert_eq!(ext("hpp"), None);
        assert_eq!(ext("txt"), None);
        assert_eq!(ext(""),    None);
    }
}
