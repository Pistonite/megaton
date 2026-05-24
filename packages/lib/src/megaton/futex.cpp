// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#include <atomic>
#include <cerrno>
#include <megaton/__internal/futex.h>
#include <optional>

extern "C" {
#include <switch/kernel/svc.h>
}

static std::optional<int64_t> to_usec(const timespec& timespec) {
    uint64_t res = 0;
    constexpr time_t USEC_IN_SEC = 1000000;
    constexpr time_t USEC_IN_NSEC = 1000;
    if (__builtin_mul_overflow(timespec.tv_sec, USEC_IN_SEC, &res)) {
        return {};
    }

    if (__builtin_add_overflow(res, timespec.tv_nsec / USEC_IN_NSEC, &res)) {
        return {};
    }
    return res;
}

extern "C" int32_t sys_futex_wake(int32_t* address, int32_t count) {
    if (address == nullptr) {
        return -EINVAL;
    }
    auto val = *address;
    return (s32)svcSignalToAddress((void*)address, SignalType_Signal, val, count);
}

extern "C" int32_t sys_futex_wait(const int32_t* address, uint32_t expected,
                                  const timespec* timeout, uint32_t flags) {
    if (address == nullptr) {
        return -EINVAL;
    }

    int64_t timeout_usec;
    std::optional<int64_t> time_usec = to_usec(*timeout);
    if (!time_usec.has_value()) {
        return -EINVAL;
    }
    timeout_usec = time_usec.value();

    uint32_t result = svcWaitForAddress((std::atomic_uint32_t*)address, ArbitrationType_WaitIfEqual,
                                        expected, timeout_usec);
    return (s32)result;
}
