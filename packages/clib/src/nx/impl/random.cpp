// This file has been modified from libnx, the project where
// it's taken from. See the license information below
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
