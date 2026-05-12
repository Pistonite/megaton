// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#include <megaton/__internal/alloc.h>
#include <nn/os/os_Mutex.h>

#include <megaton/__internal/tlsf.h>
// #include <toolkit/tcp.hpp>

static u8 bss_alloc[BSS_ALLOC_SIZE] = {0};
static nn::os::Mutex mem_mutex = nn::os::Mutex(true /* recursive */);
static pool_t allocator;
static bool mem_initialized = false;

// nn::mem::StandardAllocator& allocator() {
//     static nn::mem::StandardAllocator instance(bss_alloc, sizeof(bss_alloc));
//     return instance;
// }

namespace botw::tcp {
    void sendf(const char* format, ...);
}

extern "C" u8 *sys_malloc(u64 size, u64 align) {
    mem_mutex.Lock();
    if (!mem_initialized) {
        allocator = tlsf_create_with_pool(bss_alloc, BSS_ALLOC_SIZE);
        mem_initialized = true;
    }
    void* ptr = tlsf_memalign(allocator, align, size);
    // botw::tcp::sendf("allocating %d bytes\n", size);
    // void *ptr = allocator().Allocate(size, align);
    return (u8*) ptr;
}

extern "C" void sys_free(u8 *ptr, u64 size, u64 align) {
    mem_mutex.Lock();
    if (mem_initialized) {
        tlsf_free(allocator, ptr);
    }
    mem_mutex.Unlock();
    // botw::tcp::sendf("calling free on ptr %d with size %d and alignment
    // %d\n",
    // ptr, size, align);
    // botw::tcp::sendf("freeing ptr %p\n", ptr);
    // allocator().Free(ptr);
    // botw::tcp::sendf("successfully freed memory\n");
    // delete ptr;
}

extern "C" u8 *sys_realloc(u8 *ptr, u64 size, u64 align, u64 new_size) {
    void* new_ptr = nullptr;
    mem_mutex.Lock();
    if (mem_initialized) {
        new_ptr = tlsf_realloc(allocator, ptr, new_size);
    }
    mem_mutex.Unlock();
    return (u8*) new_ptr;
    // botw::tcp::sendf(
    //     "calling realloc on ptr %p from size %d to size %d with alignment\n",
    //     ptr, size, new_size, align);
    // if (new_size == 0) {
    //     sys_free(ptr, size, align);
    //     return nullptr;
    // }
    // u8 *new_ptr = sys_malloc(new_size, align);
    // if (new_ptr == nullptr) {
    //     return new_ptr;
    // }
    // for (u64 i = 0; i < new_size; i++) {
    //     if (i >= size) {
    //         new_ptr[i] = 0;
    //     } else {
    //         new_ptr[i] = ptr[i];
    //     }
    // }
    // sys_free(ptr, size, align);
    // return new_ptr;
}
