#include "futex.h"
#include <errno.h>

static std::optional<int64_t> to_usec(const timespec& timespec) {
    int64_t res;
    if(__builtin_mul_overflow(timespec.tv_sec, 1000000, &res)) 
        return {};
    
    if(__builtin_add_overflow(res, timespec.tv_nsec / 1000, &res))
        return {};
    return res;
}

extern "C" int32_t sys_futex_wake(uint32_t *address, int32_t count) {
    if (address == nullptr)
        return -EINVAL; // EINVAL
    int32_t val = (int32_t)*address;
    return svcSignalToAddress((void *)address, SignalType_Signal, val, count);
}

extern "C" int32_t sys_futex_wait(uint32_t *address, uint32_t expected, const timespec *timeout, uint32_t flags) {
    if (address == nullptr)
        return -EINVAL;

    std::atomic_uint32_t *atomic_address = (std::atomic_uint32_t *)address;

    int64_t timeout_usec;
    std::optional<uint64_t> t = to_usec(*timeout);
    if (!t.has_value())
        return -EINVAL;
    timeout_usec = t.value();
    if ((flags & 0b01) == 0) { // absolute timeout -- convert to relative
        uint64_t current_time_usec = 0;
        timeout_usec -= current_time_usec;
    }

    uint32_t result = svcWaitForAddress(atomic_address, ArbitrationType_WaitIfEqual, expected, timeout_usec);
    return result;
}
