#pragma once

#include <cstdint>
#include <atomic>

typedef uint32_t u32;
typedef u32 Result;
typedef int64_t s64;
typedef int32_t s32;

extern "C" Result svcSignalToAddress(void *address, u32 signal_type, s32 value, s32 count);
extern "C" Result svcWaitForAddress(void *address, u32 arb_type, s64 value, s64 timeout);

typedef enum {
    ArbitrationType_WaitIfLessThan             = 0, ///< Wait if the 32-bit value is less than argument.
    ArbitrationType_DecrementAndWaitIfLessThan = 1, ///< Decrement the 32-bit value and wait if it is less than argument.
    ArbitrationType_WaitIfEqual                = 2, ///< Wait if the 32-bit value is equal to argument.
    ArbitrationType_WaitIfEqual64              = 3, ///< [19.0.0+] Wait if the 64-bit value is equal to argument.
} ArbitrationType;

typedef enum {
    SignalType_Signal                                          = 0, ///< Signals the address.
    SignalType_SignalAndIncrementIfEqual                       = 1, ///< Signals the address and increments its value if equal to argument.
    SignalType_SignalAndModifyBasedOnWaitingThreadCountIfEqual = 2, ///< Signals the address and updates its value if equal to argument.
} SignalType;

namespace futex { 
    extern "C" int32_t sys_futex_wake(uint32_t *address, int32_t count);
    extern "C" int32_t sys_futex_wait(uint32_t *address, uint32_t expected, const timespec *timeout, uint32_t flags);
}
