// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#pragma once

#include <switch/types.h>
#define BSS_ALLOC_SIZE 0x20000

namespace nn::mem {
    class StandardAllocator {
        public:
            StandardAllocator();
            // StandardAllocator(void* address, size_t size);
            // StandardAllocator(void* address, size_t size, bool enableCache);

            ~StandardAllocator() {
                if (mIsInitialized) {
                    Finalize();
                }
            }

            void Initialize(void* address, size_t size);
            void* Allocate(size_t size);
            void* Allocate(size_t size, size_t alignment);
            void Free(void* address);
            bool mIsInitialized;
            void Finalize();
    };
}
