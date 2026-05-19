// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#include <megaton/__internal/alloc.h>
#include <nn/os/os_Mutex.h>
#include <mutex>

#include <megaton/__internal/tlsf.h>

u8 bss_alloc[BSS_ALLOC_SIZE] = {0};
static nn::os::Mutex mem_mutex = nn::os::Mutex(true /* recursive */);
pool_t allocator;

extern "C" u8 *sys_malloc(u64 size, u64 align) {
    std::lock_guard<nn::os::Mutex> lock(mem_mutex);
    void* ptr = tlsf_memalign(allocator, align, size);
    return (u8*) ptr;
}

extern "C" void sys_free(u8 *ptr, u64 size, u64 align) {
    std::lock_guard<nn::os::Mutex> lock(mem_mutex);
        tlsf_free(allocator, ptr);
}

extern "C" u8 *sys_realloc(u8 *ptr, u64 size, u64 align, u64 new_size) {
    void* new_ptr = nullptr;
    std::lock_guard<nn::os::Mutex> lock(mem_mutex);
        new_ptr = tlsf_realloc(allocator, ptr, new_size);
    return (u8*) new_ptr;
}
