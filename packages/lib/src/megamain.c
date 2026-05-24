// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#include <megaton/types.h>

/**
 * module name needs to put at nx-module-name section
 */
struct module_name_t {
    i32 unknown;
    i32 name_len;
    u8 name[MEGART_NX_MODULE_NAME_LEN + 1];
};

__attribute__((section(".nx-module-name"))) __attribute__((used))
const struct module_name_t s_module_name = {.unknown = 0,
                                            .name_len =
                                                MEGART_NX_MODULE_NAME_LEN,
                                            .name = MEGART_NX_MODULE_NAME};

// NOLINTNEXTLINE(bugprone-reserved-identifier)
const char* __megaton_module_name() { return MEGART_NX_MODULE_NAME; }

// NOLINTNEXTLINE(bugprone-reserved-identifier)
usize __megaton_module_name_len() { return MEGART_NX_MODULE_NAME_LEN; }

// NOLINTNEXTLINE(bugprone-reserved-identifier)
u64 __megaton_title_id() { return MEGART_TITLE_ID; }

// NOLINTNEXTLINE(bugprone-reserved-identifier)
const char* __megaton_title_id_hex() { return MEGART_TITLE_ID_HEX; }

// NOLINTNEXTLINE(bugprone-reserved-identifier)
void __megaton_lib_init();
// NOLINTNEXTLINE(bugprone-reserved-identifier)
void __megaton_librs_init();
void megaton_main();

// real module entry point
// NOLINTNEXTLINE(bugprone-reserved-identifier)
void __megaton_module_entry() {
    // initialize C lib
    __megaton_lib_init();

    // initialize Rust lib
#ifdef MEGART_RUST
    __megaton_librs_init();
#endif
    // bootstrap done

    // call module main
    megaton_main();
}
