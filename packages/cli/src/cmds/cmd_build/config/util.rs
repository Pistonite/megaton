// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::collections::BTreeMap;

use cu::pre::*;

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
        cu::bailfyi!("error found at config key: {}", self.key);
    }
}
