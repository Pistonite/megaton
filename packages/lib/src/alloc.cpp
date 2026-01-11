// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

#include <alloc.hpp>
#include <toolkit/tcp.hpp>

static u8 bss_alloc[BSS_ALLOC_SIZE] = {0};
static nn::mem::StandardAllocator sa = nn::mem::StandardAllocator();

extern "C" u8* sys_malloc(u64 size, u64 align) {
    botw::tcp::sendf("calling malloc for %d bytes with alignment %d\n", size, align);
    if (!sa.mIsInitialized) {
        sa.Initialize(bss_alloc, BSS_ALLOC_SIZE);
    }
    void* ptr = sa.Allocate(size, align);
    if (ptr != NULL) {
        return (u8*)ptr;
    }
    return nullptr;
}

extern "C" void sys_free(u8* ptr, u64 size, u64 align) {
    botw::tcp::sendf("calling free on ptr %d with size %d and alignment %d\n", ptr, size, align);
    if (!sa.mIsInitialized) {
        return;
    }
    sa.Free(ptr);
    botw::tcp::sendf("successfully freed memory\n");
}

extern "C" u8* sys_realloc(u8* ptr, u64 size, u64 align, u64 new_size) {
    botw::tcp::sendf("calling realloc on ptr %p from size %d to size %d with alignment\n", ptr, size, new_size, align);
    if (new_size == 0) {
        sys_free(ptr, size, align);
        return nullptr;
    }
    u8* new_ptr = sys_malloc(new_size, align);
    if (new_ptr == nullptr) {
        return new_ptr;
    }
    for (u64 i = 0; i < new_size; i++) {
        if (i >= size) {
            new_ptr[i] = 0;
        } else {
            new_ptr[i] = ptr[i];
        }
    }
    sys_free(ptr, size, align);
    return new_ptr;
}
