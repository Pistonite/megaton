// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors
// * * * * *
// Part of this file was taken from the libnx project and modified
// See original license information below
//
// SPDX-License-Identifier: ISC
// Copyright (c) libnx authors

/* 
 * Megaton C runtime initialization
 *
 * Our initialization needs to be customized, because we are neither:
 * - a NRO homebrew app using the Homebrew ABI
 * - a NSO app; The game's rtld is the main NSO app
 *
 * When rtld loads us, it jumps to the start symbol of our binary `_start`,
 * defined in (our) crt0.o. We then initialize libnx through our custom
 * __libnx_init to initialize c++ libs such as newlib, and call global constructors
 * (which involve crti/crtbegin from the toolchain - those are not custom).
 * This is __megacrt_init. And this is implemented in C so we don't accidentally
 * use c++ features before CRT is initialized.
 *
 * At this point the CRT is initialized and we can safely use c++ features.
 * crt0 jumps to _megaton_init implemented in user_rt.c. The Megaton C++ lib
 * is initialized, and optionally the Megaton Rust lib. Finally the control
 * is passed to user's megaton_main
 */

#include <stdint.h>
#include <stddef.h>
#include <switch/kernel/svc.h>
#include <switch/runtime/env.h>
#include <switch/runtime/diag.h>
// #include <switch/runtime/devices/fs_dev.h>
#include <switch/services/applet.h>
// #include <switch/result.h>
#include <sys/unistd.h>

#include <megaton/user_rt.h>
#include "megacrt.h"

void virtmemSetup(void);
void newlibSetup(void);
void argvSetup(void);
void __libnx_initheap();
void __libnx_init_thread();

// override the number of fs sessions in libnx so we are less likely to hit the process limit
u32 __nx_fs_num_sessions = 1;

/**
 * Our abort.
 *
 * If something request exit of process, it's probably best to crash the game
 */
// NOLINTNEXTLINE(bugprone-reserved-identifier)
__attribute__((noinline)) __attribute__((noreturn)) void __megacrt_abort(int rc) {
    diagAbortWithResult(rc);
    // if it doesn't crash for some reason then we try to cause a data abort
    // from: exlaunch/source/lib/diag/abort.cpp
    // this will store val to an invalid address, causing a data abort
    register int64_t addr __asm__("x27") = 0x6969696969696969;
    register int64_t val __asm__("x28") = 0x0CCCCCCCCCCCCCCC;
    while (1) {
        __asm__ __volatile__("str %[val], [%[addr]]" : : [val] "r"(val), [addr] "r"(addr));
    }
}


__attribute__((noinline)) void __megacrt_init()  {
    // called by crt0.
    // We need to first initialize homebrew environment before we can do anything
    u32 hosversion = __megacrt_get_hos_version_for_libnx();
    Handle main_thread = __megacrt_get_current_thread();
    ConfigEntry ctx[] = {
        {
            .Key = EntryType_MainThreadHandle,
            .Flags = 0,
            .Value = { main_thread , 0},
        },
        // override heap to be the fake bss heap. this is needed because
        // the game will allocate all memory in nninitStartup. If we allocate memory
        // before, then that will change how the game allocates memory, which will
        // might make singletons allocate to different addresses than expected.
        {
            .Key = EntryType_OverrideHeap,
            .Flags = 0,
            .Value = { 
                (u64)(_megart_fake_heap_start()),
                _megart_fake_heap_size()
            }
        },
        // syscalls are all made available through the overriden npdm
        {
            .Key = EntryType_SyscallAvailableHint,
            .Flags = 0,
            .Value = { UINT64_MAX, UINT64_MAX }
        },
        // this force skips applet initialization which does not apply to us
        {
            .Key = EntryType_AppletType,
            .Flags = 0,
            .Value = { AppletType_None, 0 }
        },
        // Not sure if this would suffice (it probably should) or if we need to use the IPC
        // trick to grab a real handle
        // see https://github.com/Pistonight/skyline/blob/da9a9f41ae93a859b23af354b5d9e9f8d0db5b2a/source/skyline/utils/ipc.cpp#L56
        // if needed, credit to shadowninja108
        {
            .Key = EntryType_ProcessHandle,
            .Flags = 0,
            .Value = { CUR_PROCESS_HANDLE, 0 },
        },
        {
            .Key = EntryType_HosVersion,
            .Flags = 0,
            .Value = { hosversion, 0 },
        },
        {
            .Key = EntryType_EndOfList,
            .Flags = 0,
            .Value = {0, 0}
        },
    };

    envSetup(ctx, main_thread, __megacrt_abort);
    // newlibSetup(); // <- we don't need this as the thread var initialization is already handled by RTLD
    virtmemSetup();
    // since we provided an override, this should just set the fake heap symbols for newlib
    __libnx_initheap();
    __libnx_init_thread();
    // this would just set argc to 0
    argvSetup();
    // we handle our own service initialization through the SDK instead of __appInit()
    __megacrt_init_services();
    // switch to the runtime directory, which is sdmc:/megaton/<module_name>
    // this is usually done by __appInit if we provide argv, which we don't
    chdir(_megart_cwd());
    
    // Call constructors.
    void __libc_init_array(void);
    __libc_init_array();
    //
    // // Transfer init control to user_rt
    // _megart_init();
}


// Override exit symbol in libnx
__attribute__((noinline)) void __nx_exit(int rc, void* _return_address) {
    __megacrt_abort(rc);
}
__attribute__((noinline)) __attribute__((noreturn)) void __libnx_exit(int rc) {
    __megacrt_abort(rc);
}
