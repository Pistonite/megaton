#pragma once

#include <cstdint>

namespace futex { 
    extern "C" int32_t sys_futex_wake(uint32_t *address, int32_t count);
}
