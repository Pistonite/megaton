// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

// this file declares some constants that are used in the allocator and in init
#pragma once

#include <switch/types.h>
#include <megaton/__internal/tlsf.h>

#define BSS_ALLOC_SIZE 0x20000

extern pool_t allocator;
extern u8 bss_alloc[BSS_ALLOC_SIZE];
