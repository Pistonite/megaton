// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#[rustfmt::skip]
#[path = "nn_result_gen.rs"]
mod gen_;
pub use gen_::*;

/// Number of bits for the module (lower 9 bits, 0-8)
const MODULE_BITS: u32 = 9;
/// Number of bits for the description (13 bits, 9-21)
const DESCRIPTION_BITS: u32 = 13;

const MODULE_MASK: u32 = (1 << MODULE_BITS) - 1;
const DESCRIPTION_MASK: u32 = (1 << DESCRIPTION_BITS) - 1;

/// Raw `nn::Result` value
///
/// The remaining 10 bits above the description are reserved, and are kept as-is
/// by this wrapper (i.e. only an all-zero value is success)
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct nn_Result(u32);

impl nn_Result {
    /// The success value
    pub const Ok: Self = Self(0);

    /// Wrap a raw result value
    #[inline(always)]
    pub const fn from_raw_unchecked(raw: u32) -> Self {
        Self(raw)
    }

    /// Convert an IO Error into `nn::Result`, if the Error came from an OS error.
    /// The internal value of the OS error is not checked.
    #[inline(always)]
    pub fn from_os_unchecked(value: std::io::Error) -> Option<Self> {
        Some(nn_Result::from_raw_unchecked(value.raw_os_error()? as u32))
    }

    /// Make a result from a raw module and a raw description
    ///
    /// Bits that don't fit in the module or the description are discarded
    #[inline(always)]
    pub const fn from_parts(module: u32, description: u32) -> Self {
        Self((module & MODULE_MASK) | ((description & DESCRIPTION_MASK) << MODULE_BITS))
    }

    /// Get the raw result value
    #[inline(always)]
    pub const fn to_raw(self) -> u32 {
        self.0
    }

    /// Check if the result is success
    #[inline(always)]
    pub const fn is_ok(self) -> bool {
        self.0 == Self::Ok.0
    }

    /// Check if the result is a failure
    #[inline(always)]
    pub const fn is_err(self) -> bool {
        !self.is_ok()
    }

    /// Get the module, `None` if it is not a known module
    #[inline(always)]
    pub const fn module(self) -> Option<nn_ResultModule> {
        nn_ResultModule::from_raw(self.raw_module())
    }

    /// Get the raw module code
    #[inline(always)]
    pub const fn raw_module(self) -> u32 {
        self.0 & MODULE_MASK
    }

    /// Get the description as `T`
    ///
    /// `None` if the result is not of `T`'s module, or if the description is
    /// not a known description of that module
    #[inline(always)]
    pub fn description<T: nn_ResultDescription>(self) -> Option<T> {
        if self.raw_module() != T::MODULE.to_raw() {
            return None;
        }
        T::from_raw(self.raw_description())
    }

    /// Get the raw description code
    #[inline(always)]
    pub const fn raw_description(self) -> u32 {
        (self.0 >> MODULE_BITS) & DESCRIPTION_MASK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let result = nn_Result::from_parts(
            nn_ResultModule::Fs.to_raw(),
            nn_ResultDescription_fs::PathNotFound.to_raw()
        );
        assert_eq!(result.description(), Some(nn_ResultDescription_fs::PathNotFound));
        assert_ne!(result.description(), Some(nn_ResultDescription_ncm::InvalidContentStorageBase))
    }
}
