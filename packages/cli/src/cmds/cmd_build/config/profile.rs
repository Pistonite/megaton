// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

//! Utils for managing profiles for sections in the config

use std::collections::BTreeMap;

use cu::pre::*;

use super::{Validate, ValidateCtx};

/// Name of the default profile
pub static BASE_PROFILE: &str = "none";

/// Check whether a profile name is legal
pub fn is_profile_name_allowed(name: &str) -> bool {
    BASE_PROFILE != name
}

/// Generic config section that can be extended with profiles
///
/// For example, the `[make]` section can have profiles with `[make.profiles.<name>]`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile<T: ExtendProfile> {
    /// The base profile
    #[serde(flatten)]
    base: T,
    /// The extended profiles
    #[serde(default)]
    profiles: ProfileMap<T>,
}

impl<T: ExtendProfile> Profile<T> {
    /// Get a profile by name
    ///
    /// If the name is "none", or there is no profile with that name,
    /// the base profile will be returned. Otherwise, returns the base profile
    /// extended with the profile with the given name.
    pub fn get_profile(&self, name: &str) -> T {
        let mut base = self.base.clone();
        if name != "none" {
            if let Some(profile) = self.profiles.0.get(name) {
                base.extend_profile(profile);
            }
        }
        base
    }
}

impl<T: ExtendProfile> Validate for Profile<T> {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.base.validate(ctx)?;
        self.profiles.validate_property(ctx, "profiles")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
struct ProfileMap<T: ExtendProfile>(BTreeMap<String, T>);

impl<T: ExtendProfile> Validate for ProfileMap<T> {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        for (name, config) in &self.0 {
            if !is_profile_name_allowed(name) {
                cu::error!("'{name}' is reserved and cannot be used as a profile name.");
                ctx.bail()?;
            }
            config.validate_property(ctx, name)?;
        }
        Ok(())
    }
}

/// A trait for extending a config section with a profile
pub trait ExtendProfile: Validate + Clone + std::fmt::Debug {
    /// Extend this config section with another
    fn extend_profile(&mut self, other: &Self);
}
