/*
 * This file is part of exlaunch, adapted for megaton project
 *
 * exlaunch is licensed under GPLv2
 */
#pragma once

#include <megaton/__priv/aligned_storage.h>
#include <megaton/__priv/mirror.h>

/** Make JIT memory with the given identifier and size. */
#define make_jit_(name, size)                                                  \
    namespace __jit::name {                                                    \
    alignas(PAGE_SIZE) __attribute__((                                         \
        section(".text.jit_" #name))) static const u8 code[size]{};            \
    }                                                                          \
    megaton::__priv::Jit name(__jit::name::code, size);

namespace megaton::__priv {

class Jit {

public:
    Jit(const u8* start, size_t size) : start((uintptr_t)start), _size(size) {}

    /** Initialize the JIT memory region */
    inline_member_ void init() { mirror.construct(start, _size); }

    inline_member_ void flush() { get_mirror().flush(); }
    inline_member_ uintptr_t ro_start() { return get_mirror().ro_start(); }
    inline_member_ uintptr_t rw_start() { return get_mirror().rw_start(); }
    inline_member_ uintptr_t size() { return get_mirror().size(); }

private:
    /** Read-execute memory. */
    uintptr_t start;
    size_t _size;
    /** Mapped writable memory. */
    AlignedStorage<Mirror> mirror;

    inline_member_ Mirror& get_mirror() { return mirror.reference(); }
};
} // namespace megaton::__priv
