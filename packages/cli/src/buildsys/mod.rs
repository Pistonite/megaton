// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors


mod args;
pub use args::*;
mod driver;
pub use driver::*;

mod check;
mod compile;
mod link;
mod rust;
mod lib_unpack;
use lib_unpack::unpack_megaton_lib;
mod miscfile;
