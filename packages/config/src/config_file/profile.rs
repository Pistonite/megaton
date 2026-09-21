// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

//! Utils for managing profiles for sections in the config

use std::collections::BTreeMap;
use std::path::Path;

use cu::pre::*;

use crate::config_file::{Resolve, Validate, ValidateCtx};
use crate::toolchain::ToolchainEnv;

/// Name of the base profile
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
        if name != BASE_PROFILE {
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

impl<T: Resolve + ExtendProfile> Resolve for Profile<T> {
    fn resolve(&mut self, root: &Path, toolchain: &ToolchainEnv) -> cu::Result<()> {
        cu::check!(self.base.resolve(root, toolchain), "failed to resolve config for profile \"none\"")?;
        for (name, config) in &mut self.profiles.0 {
            cu::check!(config.resolve(root, toolchain), "failed to resolve config for profile \"{name}\"")?;
        }
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

/// Extend by overiding if the extension value is some
pub fn extend_by_overriding_if_some<T: Clone>(current: &mut Option<T>, incoming_extend: Option<&T>) {
    if let Some(x) = incoming_extend {
        *current = Some(x.clone());
    }
}

/// Extend by appending the incoming values to current
pub fn extend_by_appending<T: Clone>(current: &mut Vec<T>, incoming_extend: &[T]) {
    current.extend_from_slice(incoming_extend)
}
