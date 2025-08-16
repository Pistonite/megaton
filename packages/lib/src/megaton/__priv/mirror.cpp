// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors
// * * * * *
// This file was taken from the exlaunch project and modified.
// See original license information below
//
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) shadowninja

#include <megaton/prelude.h>

#include <algorithm>
#include <cstring>

extern "C" {
#include <switch/arm/cache.h>
#include <switch/kernel/svc.h>
#include <switch/kernel/virtmem.h>
#include <switch/result.h>
}

#include <megaton/__priv/mirror.h>
#include <megaton/__priv/proc_handle.h>

namespace megaton::__priv {

/** Map or unmap the memory in the given range. */
static void handle_mapping(
    /** Page-aligned start of the read-only region. */
    uintptr_t ro_start_aligned,
    /** Page-aligned start of the read-write region. */
    uintptr_t rw_start_aligned,
    /** Page-aligned size of the region. */
    size_t size_aligned,
    /** Current process handle. */
    Handle process,
    /** Map or unmap */
    bool map) {
    const uintptr_t end_aligned = ro_start_aligned + size_aligned;

    MemoryInfo meminfo{.addr = ro_start_aligned};
    u32 pageinfo;

    do {
        // Query next range
        if (R_FAILED(svcQueryMemory(&meminfo, &pageinfo,
                                    meminfo.addr + meminfo.size))) {
            panic_("mirror: svcQueryMemory failed");
        }

        // Calculate offset into the range we are mapping.
        // Force the start to be at least the aligned start if for some reason
        // it is not
        uintptr_t offset =
            std::max(meminfo.addr, ro_start_aligned) - ro_start_aligned;
        // Determine the address we will be working on.
        uintptr_t ro_start = ro_start_aligned + offset;
        void* rw_start = (void*)(rw_start_aligned + offset);
        /* Determine the size of this range to map/unmap. */
        uintptr_t size =
            std::min(end_aligned, meminfo.addr + meminfo.size) - ro_start;

        if (map) {
            if (R_FAILED(
                    svcMapProcessMemory(rw_start, process, ro_start, size))) {
                panic_("mirror: svcMapProcessMemory failed");
            }
        } else {
            if (R_FAILED(
                    svcUnmapProcessMemory(rw_start, process, ro_start, size))) {
                panic_("mirror: svcUnmapProcessMemory failed");
            }
        }
    } while ((meminfo.addr + meminfo.size) < end_aligned);
}

Mirror::Mirror(uintptr_t start, size_t size) {
    m.ro_start = start;
    m.size = size;

    auto size_aligned = m.size_aligned();

    // Find a page for the RW region and reserve it
    uintptr_t rw_start_aligned = (uintptr_t)virtmemFindAslr(size_aligned, 0);
    assert_(rw_start_aligned != 0);
    auto reserve = virtmemAddReservation((void*)rw_start_aligned, size_aligned);
    assert_(reserve != NULL);
    m.rw_reserve = reserve;

    // Get the process handle for mapping memory
    auto process = megaton::__priv::current_process();

    auto ro_start_aligned = m.ro_start_aligned();

    handle_mapping(ro_start_aligned, rw_start_aligned, size_aligned, process,
                   true);

    // Setup RW pointer to match same unaligned location of RO.
    m.rw_start = rw_start_aligned + (start - ro_start_aligned);

    // Ensure the mapping worked
    assert_(memcmp((void*)m.ro_start, (void*)m.rw_start, size) == 0);
}

void Mirror::flush() {
    auto size_aligned = m.size_aligned();
    armDCacheFlush((void*)m.rw_start_aligned(), size_aligned);
    armICacheInvalidate((void*)m.ro_start_aligned(), size_aligned);
}

Mirror::~Mirror() {
    /* Only uninit if this is the owner. */
    if (!m.rw_reserve)
        return;

    flush();

    auto process = megaton::__priv::current_process();

    handle_mapping(m.ro_start_aligned(), m.rw_start_aligned(), m.size_aligned(),
                   process, false);

    // Free reservation of the read-write region
    virtmemRemoveReservation(m.rw_reserve);
}
}; // namespace megaton::__priv
