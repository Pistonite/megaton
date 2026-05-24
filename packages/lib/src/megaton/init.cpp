// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors
// * * * * *
// Parts of this file was taken from the exlaunch project and modified.
// See original license information below
//
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) shadowninja

#include <megaton/__internal/alloc.h>
#include <megaton/hook.h>
#include <megaton/module_layout.h>
#include <megaton/patch.h>

extern "C" {
/**
 * These magic symbols are provided by the linker script
 *
 * from exlaunch/source/lib/init/init.cpp
 */
// NOLINTNEXTLINE(bugprone-reserved-identifier) linker magic symbol
extern void (*__preinit_array_start[])(void) __attribute__((weak));
// NOLINTNEXTLINE(bugprone-reserved-identifier) linker magic symbol
extern void (*__preinit_array_end[])(void) __attribute__((weak));
// NOLINTNEXTLINE(bugprone-reserved-identifier) linker magic symbol
extern void (*__init_array_start[])(void) __attribute__((weak));
// NOLINTNEXTLINE(bugprone-reserved-identifier) linker magic symbol
extern void (*__init_array_end[])(void) __attribute__((weak));

// NOLINTNEXTLINE(bugprone-reserved-identifier) FIXME TODO can this be removed??
void __init_array(void) {
    // NOLINTBEGIN(clang-analyzer-security.PointerSub)
    // UB subtracting 2 pointers that don't point to the same array
    {
        usize count = __preinit_array_end - __preinit_array_start;
        for (usize i = 0; i < count; i++) {
            __preinit_array_start[i]();
        }
    }

    {
        usize count = __init_array_end - __init_array_start;
        for (usize i = 0; i < count; i++) {
            __init_array_start[i]();
        }
    }
    // NOLINTEND(clang-analyzer-security.PointerSub)
}

// from libnx
// virtmem needed for mapping writable memory to read-only memory
// the setup is not in libnx's header for some reason
void virtmemSetup(void);

// NOLINTNEXTLINE(bugprone-reserved-identifier) FIXME
void __megaton_lib_init() {
    virtmemSetup();
    megaton::module::init_layout();
    megaton::patch::init();

    __init_array();
    megaton::hook::init();
}

// NOLINTNEXTLINE(bugprone-reserved-identifier) FIXME
void __megaton_librs_init() {
    megaton::alloc::init_allocator();
}
// TODO: this can probably be removed with rtld/reloc
// NOLINTNEXTLINE(bugprone-reserved-identifier) FIXME
void __megaton_rtld_init() {}
}
