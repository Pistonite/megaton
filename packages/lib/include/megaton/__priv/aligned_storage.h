// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors
// * * * * *
// This file was taken from the exlaunch project and modified.
// See original license information below
//
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) shadowninja
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) Atmosphère-NX

#pragma once
#include <memory>
#include <new>
#include <utility>

#include <megaton/prelude.h>

namespace megaton::__priv {

/** Aligned storage for an object of type T. */
template <typename T, size_t Size = sizeof(T), size_t Align = alignof(T)>
class AlignedStorage {

public:
    inline_member_ T* pointer() { return std::launder(pointer_internal()); }
    inline_member_ const T* pointer() const {
        return std::launder(pointer_internal());
    }
    inline_member_ T& reference() { return *pointer(); }
    inline_member_ const T& reference() const { return *pointer(); }
    template <typename... Args> inline_member_ T* construct(Args&&... args) {
        return std::construct_at(pointer_internal(),
                                 std::forward<Args>(args)...);
    }

    inline_member_ void destroy() {
        return std::destroy_at(pointer_internal());
    }

private:
    typename std::aligned_storage<Size, Align>::type _storage;

    inline_member_ T* pointer_internal() {
        return reinterpret_cast<T*>(std::addressof(_storage));
    }
};

} // namespace megaton::__priv
