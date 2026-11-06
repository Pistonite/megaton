
// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors
// * * * * *
// 
// crt0.s
// This isn't a CRT0 in a traditional sense. It is here to put some magic
// at the beginning of the module for RTLD to parse (as defined in the linker script,
// .text.crt0 is at the beginning of .text, which is at the beginning of the module).
// 
// But it also defines the _start symbol which is the dynamic entry point (DT_INIT),
// which initializes libnx (and newlib, libc, etc, which are part of libnx). For this reason
// we still call it crt0

    .section ".text.rocrt_init","ax",%progbits
    .global __module_start
    .align 2
_module_start:
    // This lets RTLD infer that ROCRT version = 0
    .word 0
    // RTLD uses this to find the static nn::ro::detail::RoModule
    .word __nx_mod0 - _module_start

    .align 4
    .ascii "~~exlaunch uwu~~"

    // Metadata for RTLD to read about our module
    .section ".text.rocrt_info","ax",%progbits
    .align 2
__nx_mod0:
    .ascii "MOD0"
    // These symbols are defined in the linker script
    .word  __dynamic_start__         - __nx_mod0
    .word  __bss_start__             - __nx_mod0
    .word  __bss_end__               - __nx_mod0
    .word  __eh_frame_hdr_start__    - __nx_mod0
    .word  __eh_frame_hdr_end__      - __nx_mod0
    // Defined below, in bss, just some spaces for RTLD to write metadata into
    .word  _ZN2nn5rocrt10g_RoModuleE - __nx_mod0

    .section ".bss"
    .balign 0x8   // alignof(nn::ro::detail::RoModule)
_ZN2nn5rocrt10g_RoModuleE:
    .skip   0x200 // sizeof(nn::ro::detail::RoModule) = 0xd0, give some extra space

    .section ".text.crt0","ax",%progbits
    .global _start
_start:
    b __megacrt_init
