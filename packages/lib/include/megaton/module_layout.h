// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors
// * * * * *
// This file was taken from the exlaunch project and modified.
// See original license information below
//
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) shadowninja

#pragma once
#include <megaton/prelude.h>

namespace megaton::module {

/**
 * Initialize the module layout
 *
 * This searches the entire address space to find the loaded modules.
 * This is automatically called in the megaton entrypoint, before
 * user-defined megaton_main
 */
void init_layout();

/** Get the number of modules */
u32 count();

/**
 * Info about a module
 *
 * Each module layout should be text -> rodata -> data
 */
class Info {
public:
    class Range {
        friend class Info;

    public:
        inline_member_ constexpr uintptr_t end() const {
            return _start + _size;
        }
        inline_member_ constexpr uintptr_t start() const { return _start; }
        inline_member_ constexpr size_t size() const { return _size; }
        inline_member_ void set(uintptr_t start, size_t size) {
            _start = start;
            _size = size;
        }

    private:
        uintptr_t _start;
        size_t _size;
    };

    inline_member_ constexpr uintptr_t start() const { return _text.start(); }
    inline_member_ constexpr uintptr_t end() const { return _data.end(); }
    inline_member_ constexpr size_t size() const {
        return _text.size() + _rodata.size() + _data.size();
    }
    inline_member_ constexpr const Range& text() const { return _text; }
    inline_member_ constexpr const Range& rodata() const { return _rodata; }
    inline_member_ constexpr const Range& data() const { return _data; }

    inline_member_ constexpr Range& text() { return _text; }
    inline_member_ constexpr Range& rodata() { return _rodata; }
    inline_member_ constexpr Range& data() { return _data; }

private:
    Range _text;
    Range _rodata;
    Range _data;
    /* TODO: bss? */
};

/** Get the module info at the given index */
const Info& info_at(u32 index);

static constexpr u32 RTLD_MODULE_IDX = 0;
static constexpr u32 MAIN_MODULE_IDX = 1;

/** Get the module info for the main module */
inline_always_ const Info& main_info() { return info_at(MAIN_MODULE_IDX); }

/** Get the module info for the rtld module */
inline_always_ const Info& rtld_info() { return info_at(RTLD_MODULE_IDX); }

/** Get the SDK module info */
const Info& sdk_info();

/** Get the info for this module */
const Info& self_info();

} // namespace megaton::module
