/**
 * Alignment related macros
 */
#pragma once

#define align_up_(x, a) ((((uintptr_t)x) + (((uintptr_t)a)-1)) & ~(((uintptr_t)a)-1))
#define align_down_(x, a) ((uintptr_t)(x) & ~(((uintptr_t)(a)) - 1))
#define aligned_(a)      __attribute__((aligned(a)))

#define PAGE_SIZE (0x1000)
