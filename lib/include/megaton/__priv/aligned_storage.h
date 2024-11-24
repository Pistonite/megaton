/*
 * Copyright (c) Atmosphère-NX
 *
 * This program is free software; you can redistribute it and/or modify it
 * under the terms and conditions of the GNU General Public License,
 * version 2, as published by the Free Software Foundation.
 *
 * This program is distributed in the hope it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

/*
 * Adapted from exlaunch for megaton project
 */

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
