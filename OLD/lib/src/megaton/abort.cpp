// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#include <switch/types.h>

extern "C" void sys_abort() {
    u8* ptr = (u8*)0xFFFFFFFFFFFFFFFF;
    // NOLINTNEXTLINE(clang-analyzer-core.FixedAddressDereference) intended to cause crash
    u8 _should_crash = *ptr; // crash should happen here (ideally)
    // get rid of error on unused variable
    (void)_should_crash;
}
