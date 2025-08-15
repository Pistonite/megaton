
// This file has been modified from exlaunch, the project where
// it's taken from. See the license information below
//
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) shadowninja

// crt0.s - C Runtime
// I don't think this is needed, but just here to 
// make sure everything works first.
// Should be trimmed with the elf/reloc stuff if not needed

.section ".text.crt0","ax"

.macro FROM_MOD0 register_num, offset
    ldr w\register_num, [x24, #\offset]
    sxtw x\register_num, w\register_num
    add x\register_num, x\register_num, x24
.endm

.macro FUNC_RELATIVE_ASLR name, register_num, symbol
.word \symbol - .
\name:
    ldr w\register_num, [x30] //  LR
    sxtw x\register_num, w\register_num
    add x\register_num, x\register_num, x30
.endm

.global __module_start
__module_start:
    b __crt_phase_user_mode_exception
    .word __nx_mod0 - __module_start

    .align 4
    .ascii "~~exlaunch uwu~~"

__crt_phase_user_mode_exception:
    // Arguments on NSO entry:
    //   x0=zero                  | x1=main thread handle
    // Arguments on NRO entry (homebrew ABI):
    //   x0=ptr to env context    | x1=UINT64_MAX (-1 aka 0xFFFFFFFFFFFFFFFF)
    // Arguments on user-mode exception entry:
    //   x0=excpt type (non-zero) | x1=ptr to excpt context

    // Detect and handle user-mode exceptions first:
    // if (x0 != 0 && x1 != UINT64_MAX) exl_exception_entry(<inargs>);
    cmp  x0, #0
    ccmn x1, #1, #4, ne // 4 = Z
    beq  __crt_phase_get_module_info
    b __megaton_crt_abort

__crt_phase_get_module_info:
    // Get start of our module
    // x23 = __module_start
    bl __get_module_start_shim
    FUNC_RELATIVE_ASLR __get_module_start_shim, 23, __module_start
    
    // Get location of MOD0
    mov x24, xzr
    ldr w24, [x23, #4]
    add x24, x24, x23

    // Get BSS regions from MOD0
    FROM_MOD0 8, 0x8
    FROM_MOD0 9, 0xC

__crt_phase_clear_bss:
    // Calculate BSS address/size
    sub  x9, x9, x8        // calculate BSS size
    add  x9, x9, #7        // round up to 8
    bic  x9, x9, #7        // ^

    // Clear the BSS in 8-byte units
bss_loop:
    subs w9, w9, #8
    str  xzr, [x8], #8
    bne  bss_loop

    // Preserve registers across function calls
    mov x25, x0  // entrypoint argument 0
    mov x26, x1  // entrypoint argument 1

    // Parse ELF .dynamic section (which applies relocations to our module)
    mov x0, x23
    FROM_MOD0 1, 0x4
    // bl   exl_dynamic

    mov  x0, x25
    mov  x1, x26
    b __megaton_rtld_init

    // failsafe
    .word 0xdeadbeef


.section ".rodata.mod0","a"

.hidden __megaton_nx_module_runtime 

.align 2
__nx_mod0:
    .ascii "MOD0"
    .word  __dynamic_start__        - __nx_mod0
    .word  __bss_start__            - __nx_mod0
    .word  __bss_end__              - __nx_mod0
    .word  __eh_frame_hdr_start__   - __nx_mod0
    .word  __eh_frame_hdr_end__     - __nx_mod0
    .word  __megaton_nx_module_runtime    - __nx_mod0
