#pragma once
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Get the start address of the BSS heap */
char* _megart_fake_heap_start();
/** Get the size of the BSS heap */
size_t _megart_fake_heap_size();
/** Get the module name, as defined in Megaton.toml module.name field */
const char* _megart_module_name();
/** Get the length of module name (without null terminator) */
size_t _megart_module_name_len();
/** Get the path to the current working directory */
const char* _megart_cwd();
/** Get the title ID, as defined in Megaton.toml module.title-id field */
uint64_t _megart_title_id();
/** Get the title ID as hex literal without the leading 0x prefix, and padded to 16 characters */
const char* _megart_title_id_hex();

/** Megaton entry point that initializes the library and hands off to user defined entry point */
void _megart_init();

#ifdef __cplusplus
}
#endif
