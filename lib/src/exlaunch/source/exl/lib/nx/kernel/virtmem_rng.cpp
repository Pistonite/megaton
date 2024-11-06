#include <random>

#include <switch/kernel/svc.h>

extern "C" uintptr_t __virtmem_rng(void) {
    std::mt19937_64 random { svcGetSystemTick() };
    return random();
}
