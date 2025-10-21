#pragma once

#include <cstdint>
#include <atomic>
#include <optional>
extern "C" {
    #include <switch/kernel/svc.h>
}

namespace futex { 
    extern "C" int32_t sys_futex_wake(uint32_t *address, int32_t count);
    extern "C" int32_t sys_futex_wait(uint32_t *address, uint32_t expected, const timespec *timeout, uint32_t flags);
}
