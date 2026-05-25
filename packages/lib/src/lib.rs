// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

// see src/megaton/futex.cpp
static_assertions::assert_type_eq_all!(hermit_abi::time_t, i64);
static_assertions::assert_eq_size!(hermit_abi::timespec, [u8; 0x10]);
static_assertions::assert_eq_size!(hermit_abi::stat, [u8; 0x78]);

mod fs;
pub use megaton_macros::main;
