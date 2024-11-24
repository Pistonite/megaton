#include <megaton/module_layout.h>
#include <megaton/patch.h>

#include <exl/hook/lib.hpp>

extern "C" {
/**
 * These magic symbols are provided by the linker script
 *
 * from exlaunch/source/lib/init/init.cpp
 */
extern void (*__preinit_array_start[])(void) __attribute__((weak));
extern void (*__preinit_array_end[])(void) __attribute__((weak));
extern void (*__init_array_start[])(void) __attribute__((weak));
extern void (*__init_array_end[])(void) __attribute__((weak));

void __init_array(void) {
    usize count;
    usize i;

    count = __preinit_array_end - __preinit_array_start;
    for (i = 0; i < count; i++)
        __preinit_array_start[i]();

    count = __init_array_end - __init_array_start;
    for (i = 0; i < count; i++)
        __init_array_start[i]();
}

// from libnx
// virtmem needed for mapping writable memory to read-only memory
// the setup is not in libnx's header for some reason
void virtmemSetup(void);

void __megaton_lib_init() {
    virtmemSetup();
    megaton::module::init_layout();
    megaton::patch::init();

    __init_array();
    exl::hook::Initialize();
}

// TODO: this can probably be removed with rtld/reloc
void __megaton_rtld_init() {}
}
