#pragma once
#include <megaton/prelude.h>
#include <megaton/align.h>


#ifndef EXL_JIT_SIZE
#define EXL_JIT_SIZE 0x1000
#endif

#ifndef EXL_INLINE_POOL_SIZE
#define EXL_INLINE_POOL_SIZE 0x1000
#endif

namespace exl::setting {

    /* How large the JIT area will be for hooks. */
    constexpr usize JitSize = EXL_JIT_SIZE;

    /* How large the area will be inline hook pool. */
    constexpr usize InlinePoolSize = EXL_INLINE_POOL_SIZE;

    /* Sanity checks. */
    static_assert(align_up_(JitSize, PAGE_SIZE) == JitSize, "JitSize is not aligned");
    static_assert(align_up_(InlinePoolSize, PAGE_SIZE) == JitSize, "InlinePoolSize is not aligned");
}
