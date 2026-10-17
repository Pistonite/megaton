#include <megaton/user_rt.h>

// This file is compiled from values given in user's Megaton.toml.
// The defines here are injected by the megaton CLI.

struct module_name_t {
    int32_t unknown;
    int32_t name_len;
    char name[MEGART_NX_MODULE_NAME_LEN + 1];
};
/** module name needs to put at nx-module-name section according to the libnx linker script */
__attribute__((section(".nx-module-name"))) __attribute__((used))
const struct module_name_t S_NX_MODULE_NAME = {
    .unknown = 0,
    .name_len = MEGART_NX_MODULE_NAME_LEN,
    .name = MEGART_NX_MODULE_NAME
};

static char S_FAKE_HEAP[MEGART_HEAP_SIZE] = {0};
char* _megart_fake_heap_start() {
    return S_FAKE_HEAP;
}
size_t _megart_fake_heap_size() {
    return MEGART_HEAP_SIZE;
}
const char* _megart_module_name() {
    return MEGART_NX_MODULE_NAME;
}
const char* _megart_cwd() {
    return MEGART_CWD;
}
size_t _megart_module_name_len() {
    return MEGART_NX_MODULE_NAME_LEN;
}
uint64_t _megart_title_id() {
    return MEGART_TITLE_ID;
}
const char* _megart_title_id_hex() {
    return MEGART_TITLE_ID_HEX;
}

// private init functions, will be called by _module_entry below
void _megaton_lib_init();
void _megaton_lib_init_rust();

// user defined entry point
void megaton_main();

// real module entry point passed to the linker
void _megart_init() {
    _megaton_lib_init();
#ifdef MEGART_RUST
    _megaton_lib_init_rust();
#endif
    megaton_main();
}
