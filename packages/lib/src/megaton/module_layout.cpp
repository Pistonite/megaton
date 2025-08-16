// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

#include <array>
extern "C" {
#include <switch/kernel/svc.h>
#include <switch/result.h>
}
#include <megaton/__priv/rtld.h>
#include <megaton/module_layout.h>

extern "C" {
/** RTLD injects info into this */
__attribute__((section(".bss"))) rtld::ModuleObject __megaton_nx_module_runtime;
}

namespace megaton::module {

static constexpr u32 MAX_MODULES = 13;
static std::array<Info, MAX_MODULES> s_info_array;
static u32 s_count = 0;
static u32 s_self_idx = MAX_MODULES + 1;

u32 count() { return s_count; }

const Info& info_at(u32 index) {
    assert_(index < s_count);
    return s_info_array[index];
}
const Info& sdk_info() {
    // SDK is always placed last in our impl
    return s_info_array[s_count - 1];
}
const Info& self_info() { return s_info_array[s_self_idx]; }

/* Provided by linker script, the start of our executable. */
extern "C" {
extern char __module_start;
}

/*
 * This initialization is adapted from exlaunch
 */
void init_layout() {
    enum class State {
        /** Looking for code (text) section. */
        Text,
        /** Expecting rodata section. */
        Rodata,
        /** Expecting data section. */
        Data,
    } state = State::Text;

    MemoryInfo meminfo{};
    u32 pageinfo;
    u32 next_id = 0;
    uintptr_t offset = 0;
    uintptr_t prev_offset = 0;
    Info builder{};

    do {
        if (MAX_MODULES <= next_id) {
            panic_("init_layout: too many static modules");
        }

        prev_offset = offset;

        // Query next range.
        if (R_FAILED(svcQueryMemory(&meminfo, &pageinfo,
                                    meminfo.addr + meminfo.size))) {
            panic_("init_layout: svcQueryMemory failed");
        }

        u32 memtype = meminfo.type & MemState_Type;
        offset = meminfo.addr;

        switch (state) {
        case State::Text: {
            if (memtype != MemType_CodeStatic || meminfo.perm != Perm_Rx) {
                // No module here, keep going...
                continue;
            }

            builder.text().set(meminfo.addr, meminfo.size);
            state = State::Rodata;
            break;
        }
        case State::Rodata: {
            if (memtype != MemType_CodeStatic || meminfo.perm != Perm_R) {
                /* Not a proper module, reset. */
                state = State::Text;
                continue;
            }

            builder.rodata().set(meminfo.addr, meminfo.size);
            state = State::Data;
            break;
        }
        case State::Data: {
            if (memtype != MemType_CodeMutable || meminfo.perm != Perm_Rw) {
                /* Not a proper module, reset. */
                state = State::Text;
                continue;
            }

            builder.data().set(meminfo.addr, meminfo.size);

            if (builder.start() == (uintptr_t)&__module_start) {
                s_self_idx = next_id;
            }

            // Store built module info.
            s_info_array[next_id++] = builder;

            // Back to initial state.
            state = State::Text;
            break;
        }
        default: {
            unreachable_();
        }
        }

        // Exit once we've wrapped the address space.
    } while (offset >= prev_offset);

    s_count = next_id;
    // Ensure we found a valid self index and module count.
    assert_(s_self_idx < s_count);
}

} // namespace megaton::module
