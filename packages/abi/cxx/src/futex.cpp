#include "futex.h"
#include <switch/kernel/svc.h> // libnx
#include <errno.h>
#include <atomic>

extern "C" int32_t sys_futex_wake(uint32_t *address, int32_t count) {
    if (address == NULL)
        return -EINVAL; // EINVAL
    int32_t val = (int32_t)*address;
    return svcSignalToAddress((void *)address, SignalType_Signal, val, count);
}

int to_usec(const timespec *timespec, uint64_t *out) {
    // Source: https://github.com/hermit-os/kernel/blob/0fa55cc454e5bc0fec38e963563114acf4e9265a/src/time.rs#L65
    // TODO: Check for overflow
    *out = (uint64_t)timespec->tv_sec * 1000000;
    *out += timespec->tv_nsec / 1000;
    return 1; // return 1 on succes, 0 on error
}

extern "C" int32_t sys_futex_wait(uint32_t *address, uint32_t expected, const timespec *timeout, uint32_t flags) {
    if (address == NULL)
        return -EINVAL;

    std::atomic_uint32_t *atomic_address = (std::atomic_uint32_t *)address;

    uint64_t timeout_usec;
    if (!to_usec(timeout, &timeout_usec))
    { // todo: replace with timespec2nsec from devkitpro
        return -EINVAL;
    }
    if (!(flags & 0b01))
    { // absolute timeout -- convert to relative
        uint64_t current_time_usec = 0;
        timeout_usec -= current_time_usec;
    }

    uint32_t result = svcWaitForAddress(atomic_address, ArbitrationType_WaitIfEqual, expected, (int64_t)timeout_usec);
    return result;
}
