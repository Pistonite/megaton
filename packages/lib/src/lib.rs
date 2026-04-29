// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

mod fs;

unsafe extern "C" fn __megaton_librs_init() {
    fs::init_stdio();
}