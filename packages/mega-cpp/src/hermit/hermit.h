// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

/** Binding for hermit kernel implementation in Rust to call into the C side */

#pragma once

#ifdef __cplusplus
#include <cstdint>
#else
#include <stdint.h>
#endif

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void _megahermit_abort();
uint32_t _megahermit_stub1();
void _megahermit_stub2();

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
