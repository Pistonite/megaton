// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use cu::pre::*;

pub struct Environment {
    megaton_home: PathBuf,
}
impl Environment {
    fn new(megaton_home: PathBuf) -> Self {
        Self { megaton_home }
    }
    /// Get the home of the megaton cache directory
    pub fn home(&self) -> &Path {
        &self.megaton_home
    }
}

static ENVIRONMENT: OnceLock<Environment> = OnceLock::new();

pub fn environment() -> &'static Environment {
    ENVIRONMENT.get().expect("environment was not initialized")
}

pub fn commit() -> &'static str {
    env!("MEGATON_COMMIT")
}

/// Initialize the environment
///
/// # Safety
/// Only safe to call when only one thread exists
pub unsafe fn init_env() -> cu::Result<()> {
    let megaton_home = std::env::var("MEGATON_HOME").unwrap_or_default();
    let megaton_home = if megaton_home.is_empty() {
        cu::debug!("MEGATON_HOME not specified, using default path ~/.cache/megaton");
        let mut home = std::env::home_dir().context("failed to get user's home directory")?;
        home.extend([".cache", "megaton"]);
        home.normalize()?
    } else {
        Path::new(&megaton_home).normalize()?
    };

    cu::debug!("megaton_home: {}", megaton_home.display());

    let env = Environment::new(megaton_home);
    if ENVIRONMENT.set(env).is_err() {
        cu::bail!("unexpected: environment was already set before init_env()");
    }

    Ok(())
}
