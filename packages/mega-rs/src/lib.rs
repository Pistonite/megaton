// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

mod hermit;

#[path = "megacpp_sys_gen.rs"]
#[allow(unused)]
#[allow(non_camel_case_types)]
mod megacpp_sys;

pub use mega_rspmacro::main;
mod nn;
