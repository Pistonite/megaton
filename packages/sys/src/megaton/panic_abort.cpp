// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

#include <cstdio>
#include <megaton/prelude.h>

extern "C" {

static panic_hook_t s_panic_hooks[32] = {nullptr};
static u32 s_panic_hooks_len = 0;

bool __megaton_add_panic_hook(panic_hook_t hook) {
    if (s_panic_hooks_len >= 32) {
        return false;
    }
    s_panic_hooks[s_panic_hooks_len++] = hook;
    return true;
}

/** Trigger data abort */
noreturn_ __megaton_abort(void) {
    // from: exlaunch/source/lib/diag/abort.cpp
    // this will store val to an invalid address, causing a data abort
    register s64 addr __asm__("x27") = 0x6969696969696969;
    register s64 val __asm__("x28") = 0x00DEAD0000DEAD00;
    while (true) {
        __asm__ __volatile__("str %[val], [%[addr]]"
                             :
                             : [val] "r"(val), [addr] "r"(addr));
    }
}

/** Trigger abort from CRT */
noreturn_ __megaton_crt_abort(void) {
    register s64 addr __asm__("x27") = 0x6969696969696969;
    register s64 val __asm__("x28") = 0xCCCCCCCCCCCCCCCC;
    while (true) {
        __asm__ __volatile__("str %[val], [%[addr]]"
                             :
                             : [val] "r"(val), [addr] "r"(addr));
    }
}

noreturn_ __megaton_handle_panic(const char* file, u32 line, const char* msg) {
    char buffer[1024];
    buffer[0] = '\0';
    snprintf(buffer, sizeof(buffer), "panic at %s:%d:\n  %s", file, line, msg);
    buffer[sizeof(buffer) - 1] = '\0';

    for (u32 i = 0; i < s_panic_hooks_len; i++) {
        auto hook = s_panic_hooks[i];
        if (hook != nullptr) {
            hook(buffer);
        }
    }
    __megaton_abort();
}

noreturn_ __megaton_handle_panic_nx_result(const char* file, u32 line,
                                           const char* msg, u32 result) {
    char nx_result_msg[256];
    nx_result_msg[0] = '\0';
    snprintf(nx_result_msg, sizeof(nx_result_msg), "%s (nx result 0x%08x)", msg,
             result);
    nx_result_msg[sizeof(nx_result_msg) - 1] = '\0';

    __megaton_handle_panic(file, line, nx_result_msg);
}
}
