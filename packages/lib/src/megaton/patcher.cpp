// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

extern "C" {
#include <switch/arm/cache.h>
}

#include <megaton/__priv/aligned_storage.h>
#include <megaton/module_layout.h>
#include <megaton/patch.h>

namespace megaton::patch {
static __priv::AlignedStorage<__priv::Mirror> s_main_rx;

const __priv::Mirror& main_ro() { return s_main_rx.reference(); }

void init() {
    auto& mod = module::main_info();
    // map the text and rodata sections
    auto start = mod.start();
    auto size = mod.text().size() + mod.rodata().size();
    s_main_rx.construct(start, size);
}

void Stream::flush() {
    if (rw_start_addr == rw_current_addr) {
        // didn't write anything, skip
        return;
    }
    // find the region to flush
    void* ro = (void*)ro_start_addr;
    void* rw = (void*)rw_start_addr;
    auto size = rw_current_addr - rw_start_addr;

    /* Flush data/instructions. */
    armDCacheFlush(rw, size);
    armICacheInvalidate(ro, size);

    // Reset start position for next flush
    rw_start_addr += size;
    ro_start_addr += size;
}

} // namespace megaton::patch
