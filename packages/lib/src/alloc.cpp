// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#include <megaton/__internal/alloc.h>
#include <nn/os/os_Mutex.h>

#include <megaton/__internal/tlsf.h>

static u8 bss_alloc[BSS_ALLOC_SIZE] = {0};
static nn::os::Mutex mem_mutex = nn::os::Mutex(true /* recursive */);
static pool_t allocator;
static bool mem_initialized = false;

extern "C" u8 *sys_malloc(u64 size, u64 align) {
    mem_mutex.Lock();
    if (!mem_initialized) {
        allocator = tlsf_create_with_pool(bss_alloc, BSS_ALLOC_SIZE);
        mem_initialized = true;
    }
    void* ptr = tlsf_memalign(allocator, align, size);
    return (u8*) ptr;
}

extern "C" void sys_free(u8 *ptr, u64 size, u64 align) {
    mem_mutex.Lock();
    if (mem_initialized) {
        tlsf_free(allocator, ptr);
    }
    mem_mutex.Unlock();
}

extern "C" u8 *sys_realloc(u8 *ptr, u64 size, u64 align, u64 new_size) {
    void* new_ptr = nullptr;
    mem_mutex.Lock();
    if (mem_initialized) {
        new_ptr = tlsf_realloc(allocator, ptr, new_size);
    }
    mem_mutex.Unlock();
    return (u8*) new_ptr;
}
