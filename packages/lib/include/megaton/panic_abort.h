// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

/**
 * panic and abort
 */
#pragma once
#include <megaton/prelude.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Panic entry point */
noreturn_ __megaton_handle_panic(const char* file, u32 line, const char* msg);
noreturn_ __megaton_handle_panic_nx_result(const char* file, u32 line,
                                           const char* msg, u32 result);

typedef void (*panic_hook_t)(const char* msg);

/**
 * Add a panic hook to be called when a panic is triggered,
 * but before aborting. Returns if the hook was added successfully.
 * Currently a maximum of 32 hooks can be added.
 */
bool __megaton_add_panic_hook(panic_hook_t hook);

#ifdef __cplusplus
}
#endif

#define panic_(msg)                                                            \
    do {                                                                       \
        __megaton_handle_panic(__FILE__, __LINE__, msg);                       \
    } while (0)

#define unreachable_() panic_("unreachable")

#define assert_(expr)                                                          \
    do {                                                                       \
        if (!bool(expr)) {                                                     \
            __megaton_handle_panic(__FILE__, __LINE__,                         \
                                   "assertion failed: " #expr);                \
        }                                                                      \
    } while (0)

#define panic_nx_(msg, result)                                                 \
    do {                                                                       \
        __megaton_handle_panic_nx_result(__FILE__, __LINE__, msg, result);     \
    } while (0)
