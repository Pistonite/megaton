// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

// Configuration corresponds to
// https://megaton-new.pistonite.dev/reference/configuration/

mod build_config;
pub use build_config::*;
mod build_flag;
pub use build_flag::*;
mod main_config;
pub use main_config::*;

mod profile;
use profile::*;
pub use profile::BASE_PROFILE;
mod util;
use util::*;
