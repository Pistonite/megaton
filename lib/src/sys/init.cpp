#include <megaton/prelude.h>

#include <exl/lib/nx/kernel/virtmem_setup.h>

#include <exl/lib/util/sys/mem_layout.hpp>
#include <exl/lib/patch/patcher_impl.hpp>
#include <exl/lib/hook/base.hpp>

extern "C" {
    /** 
     * These magic symbols are provided by the linker script
     *
     * from exlaunch/source/lib/init/init.cpp
     */
    extern void (*__preinit_array_start []) (void) __attribute__((weak));
    extern void (*__preinit_array_end []) (void) __attribute__((weak));
    extern void (*__init_array_start []) (void) __attribute__((weak));
    extern void (*__init_array_end []) (void) __attribute__((weak));

    void __init_array(void) {
        usize count;
        usize i;

        count = __preinit_array_end - __preinit_array_start;
        for (i = 0; i < count; i++)
            __preinit_array_start[i] ();

        count = __init_array_end - __init_array_start;
        for (i = 0; i < count; i++)
            __init_array_start[i] ();
    }

    void __megaton_lib_init() {
        exl::util::impl::InitMemLayout();
        virtmemSetup();
        exl::patch::impl::InitPatcherImpl();

        __init_array();
    exl::hook::Initialize();
    }

}
