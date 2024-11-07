use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::hintln;

/// Container for detecting and warning user about unused values
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unused(BTreeMap<String, toml::Value>);

impl Unused {
    pub fn check(&self) {
        if !self.0.is_empty() {
            for key in self.0.keys() {
               hintln!("Warning", "config `{}` is unused", key);
            }
        }
    }
    pub fn check_prefixed(&self, prefix: &str) {
        if !self.0.is_empty() {
            for key in self.0.keys() {
               hintln!("Warning", "config `{}.{}` is unused", prefix, key);
            }
            
        }
    }
}
