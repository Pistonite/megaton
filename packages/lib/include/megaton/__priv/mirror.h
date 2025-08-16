// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

// This file has been modified from exlaunch, the project where
// it's taken from. See the license information below
//
#pragma once
#include <megaton/align.h>
#include <megaton/prelude.h>

#include <utility>

namespace megaton::__priv {
/**
 * Writable memory mapped to read-only memory,
 * to facilitate patching
 */
class Mirror {
private:
    class Info {
        friend class Mirror;
        /** Start of the read-only region. */
        uintptr_t ro_start = 0;
        /** Start of the read-write region. */
        uintptr_t rw_start = 0;
        /** Size of the region. */
        size_t size = 0;
        /** Reservation for the read-write region. */
        struct VirtmemReservation* rw_reserve = nullptr;

        /** Page-aligned start of the read-only region. */
        constexpr uintptr_t ro_start_aligned() const {
            return align_down_(ro_start, PAGE_SIZE);
        }

        /** Page-aligned start of the read-write region. */
        constexpr uintptr_t rw_start_aligned() const {
            return align_down_(rw_start, PAGE_SIZE);
        }

        /** Page-aligned size of the region. */
        constexpr size_t size_aligned() const {
            return align_up_(size, PAGE_SIZE);
        }
    };

public:
    /** Map writable memory to the (read-only) region with start and size */
    Mirror(uintptr_t start, size_t size);
    ~Mirror();

    /* Not copyable. */
    Mirror(const Mirror&) = delete;
    Mirror& operator=(const Mirror&) = delete;

    /* Explicitly only allow moving. */
    Mirror(Mirror&& other) : m(std::exchange(other.m, {})) {}
    Mirror& operator=(Mirror&& other) {
        m = std::exchange(other.m, {});
        return *this;
    }

    /** Flush written changes to physical memory. */
    void flush();

    /** Get the start of the read-only region. */
    inline uintptr_t ro_start() const { return m.ro_start; }
    /** Get the start of the read-write region. */
    inline uintptr_t rw_start() const { return m.rw_start; }
    /** Get the size of the region. */
    inline uintptr_t size() const { return m.size; }

private:
    inline const Info& info() const { return m; }

    Info m;
};
}; // namespace megaton::__priv
