// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#include <array>
#include <mutex>

#include <nn/os/os_Mutex.h>

#include <megaton/__external/tlsf.h>
#include <megaton/__internal/alloc.h>

static constexpr size_t BSS_ALLOC_SIZE = 0x20000;
static std::array<u8, BSS_ALLOC_SIZE> bss_alloc{0};
static nn::os::Mutex ALLOCATOR_MUTEX = nn::os::Mutex(true /* recursive */);
static pool_t ALLOCATOR;

namespace megaton::alloc {
void init_allocator() {
    ALLOCATOR = tlsf_create_with_pool(bss_alloc.data(), bss_alloc.size());
}
} // namespace megaton::alloc

extern "C" u8* sys_malloc(u64 size, u64 align) {
    std::scoped_lock<nn::os::Mutex> lock(ALLOCATOR_MUTEX);
    void* ptr = tlsf_memalign(ALLOCATOR, align, size);
    return (u8*)ptr;
}

extern "C" void sys_free(u8* ptr, u64 size, u64 align) {
    std::scoped_lock<nn::os::Mutex> lock(ALLOCATOR_MUTEX);
    tlsf_free(ALLOCATOR, ptr);
}

extern "C" u8* sys_realloc(u8* ptr, u64 size, u64 align, u64 new_size) {
    void* new_ptr = nullptr;
    std::scoped_lock<nn::os::Mutex> lock(ALLOCATOR_MUTEX);
    new_ptr = tlsf_realloc(ALLOCATOR, ptr, new_size);
    return (u8*)new_ptr;
}
