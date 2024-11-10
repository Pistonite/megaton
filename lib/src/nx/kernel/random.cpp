#include <random>

extern "C" {
#include <switch/kernel/svc.h>
    /**
     * Generate a random number for virtmem
     *
     * Credit: exlaunch/source/lib/util/random.cpp
     */
    uintptr_t __libnx_virtmem_rng(void) {
        std::mt19937_64 random { svcGetSystemTick() };
        return random();
    }
}
