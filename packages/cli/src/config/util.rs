// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use std::collections::BTreeMap;

use cu::pre::*;
use semver::{Version, VersionReq};

/// Container for detecting and warning user about unused values
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureUnused(BTreeMap<String, toml::Value>);

impl Validate for CaptureUnused {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        for key in self.0.keys() {
            cu::warn!("config property {}.{} is unused", ctx.key(), key);
        }
        Ok(())
    }
}

/// Trait for validating the config
pub trait Validate {
    /// Validate the config object, as the `key` property in the parent object
    fn validate_property(&self, ctx: &mut ValidateCtx, key: &str) -> cu::Result<()> {
        ctx.push(key);
        let x = self.validate(ctx);
        ctx.pop();
        x
    }

    fn validate_root(&self) -> cu::Result<()> {
        let mut ctx = ValidateCtx::default();
        self.validate(&mut ctx)
    }

    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()>;
}

#[derive(Default)]
pub struct ValidateCtx {
    key: String,
    len_stack: Vec<usize>,
}
impl ValidateCtx {
    /// Pop the last key path
    fn pop(&mut self) {
        match self.len_stack.pop() {
            None => self.key.clear(),
            Some(i) => self.key.truncate(i),
        }
    }

    /// Push a new key segment to the path
    fn push(&mut self, key: &str) {
        self.len_stack.push(self.key.len());
        if self.key.is_empty() {
            self.key.push_str(key)
        } else {
            use std::fmt::Write;
            write!(self.key, ".{key}").unwrap();
        }
    }

    /// Get the current key path being validated
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn bail(&self) -> cu::Result<()> {
        cu::bail!("error found at config key: {}", self.key);
    }
}

/// Check the current buildtool/library version matches the requirement
pub fn check_megaton_version_requirement(version_req: &VersionReq) -> cu::Result<()> {
    let curr_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    if !version_req.matches(&curr_version) {
        cu::bail!("the project requires megaton version {version_req}, current version is {curr_version}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_pkg_version_is_valid_semver() -> cu::Result<()> {
        Version::parse(env!("CARGO_PKG_VERSION"))?;
        Ok(())
    }

    #[test]
    fn version_req_wildcard_matches() -> cu::Result<()> {
        let req = VersionReq::parse("*")?;
        assert!(check_megaton_version_requirement(&req).is_ok());
        Ok(())
    }

    #[test]
    fn version_req_impossible_fails() -> cu::Result<()>{
        let curr = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let req = VersionReq::parse(">=99999.0.0")?;
        let result = check_megaton_version_requirement(&req);
        assert!(result.is_err());
        let error_message = format!("{:?}", result.unwrap_err());
        assert_eq!(
        error_message,
            format!("the project requires megaton version >=99999.0.0, current version is {curr}")
        );
        Ok(())
    }
}

