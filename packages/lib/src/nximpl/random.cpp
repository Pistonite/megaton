// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors
// * * * * *
// This file was taken from the exlaunch project and modified.
// See original license information below
//
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) shadowninja

#include <random>

extern "C" {
#include <switch/kernel/svc.h>
    /**
     * Generate a random number for virtmem
     */
    uintptr_t __libnx_virtmem_rng(void) {
        std::mt19937_64 random { svcGetSystemTick() };
        return random();
    }
}
