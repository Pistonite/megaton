#pragma once

#include <string_view>
#include <cstdint>

namespace megaton {

/** Name of the module, as specified in Megaton.toml module.name field */
std::string_view module_name();
/** Title ID as specified in Megaton.toml module.title-id field */
uint64_t title_id();
/** Title ID as hex string (without the 0x prefix), as specified in Megaton.toml module.title-id field */
std::string_view title_id_hex();



namespace runtime {

/**
 * Implementation of megaton's __libnx_init
 *
 * This does everything the default __libnx_init does, except for:
 * - argvSetup(), as a game mod doesn't take any arguments
 * - __appInit(), as the services are probably already initialized by the game,
 *   we don't want to initialize the services again.
 *
 * Note we also override __libnx_initheap to use a static BSS heap, since the
 * game usually allocates all heap memory on start up.
 *
 * If for some reason you need to override __libnx_init, you can call this
 * inside your implementation to potentially reuse some logic
 */
void libnx_init(uint32_t main_thread_handle);

/**
 * Implementation of megaton's __libnx_initheap
 *
 * This sets the fake_heap_start and fake_heap_end symbols for newlib
 * to point to the static BSS heap
 */
void libnx_initheap();

}


}
