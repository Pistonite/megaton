// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

#include <toolkit/tcp.hpp>
#include <switch/types.h>

extern "C" void init_env();

extern "C" void sys_abort() {
    init_env();
    botw::tcp::sendf("aborting due to panic in new library!\n");
    u8* ptr = (u8*) 0xFFFFFFFFFFFFFFFF;
    u8 x = *ptr; // crash should happen here (ideally)
    // get rid of error on unused variable
    (void) x;
    return;
}
