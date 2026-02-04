// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

/**
 * Header intended to be included by the project
 * for module info.
 */
#pragma once
#include <megaton/prelude.h>

#ifdef __cplusplus
extern "C" {
#endif

extern const char* __megaton_module_name(void);
extern usize __megaton_module_name_len(void);
extern const char* __megaton_title_id_hex(void);
extern u64 __megaton_title_id(void);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
namespace megaton {

/**
 * Get the module name.
 */
inline_always_ const char* module_name() { return __megaton_module_name(); }

/**
 * Get the module name length.
 */
inline_always_ usize module_name_len() { return __megaton_module_name_len(); }

/**
 * Get the title ID in hexadecimal, without the 0x prefix
 */
inline_always_ const char* title_id_hex() { return __megaton_title_id_hex(); }

/**
 * Get the title ID as a 64-bit integer.
 */
inline_always_ u64 title_id() { return __megaton_title_id(); }

} // namespace megaton
#endif
