// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

mod source_type;
use source_type::*;

mod compile_db;
use compile_db::*;
mod source;
use source::SourceStatus;
mod driver;
pub use driver::{CompileCtx, compile_all};
