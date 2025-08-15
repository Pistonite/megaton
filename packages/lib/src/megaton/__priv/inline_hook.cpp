// This file has been modified from exlaunch, the project where
// it's taken from. See the license information below
//
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) shadowninja

#include <array>
#include <utility>

#include <exl_armv8/prelude.h>

#include <megaton/__priv/jit.h>
#include <megaton/align.h>
#include <megaton/hook.h>
#include <megaton/module_layout.h>

/** Size of inline JIT memory. */
#ifndef MEGATON_INLINE_JIT_SIZE
#define MEGATON_INLINE_JIT_SIZE 0x1000
#endif
static_assert(align_up_(MEGATON_INLINE_JIT_SIZE, PAGE_SIZE) ==
                  MEGATON_INLINE_JIT_SIZE,
              "MEGATON_INLINE_JIT_SIZE is not page-aligned");

namespace megaton::__priv {
make_jit_(s_inline_jit_pool, MEGATON_INLINE_JIT_SIZE);

/** Initialize the inline hook system */
void init_inline_hook() { s_inline_jit_pool.init(); }

/**
 * Entry in the inline hook pool
 *
 * An inline hook is basically a trampoline hook, with the hook
 * implemented in assembly to backup and restore all registers
 */
struct InlineHookEntry {
    /** Implementation of the underlying trampoline hook */
    std::array<exl::armv8::inst::Insn, 4> trampoline;
    /**
     * User-defined callback to inspect the context
     *
     * This should be a void(*)(InlineCtx*) function
     */
    uintptr_t inline_hook_callback;
};

static constexpr size_t TOTAL =
    MEGATON_INLINE_JIT_SIZE / sizeof(InlineHookEntry);

/* Size of stack to reserve for the context. Adjust this along with
 * CTX_STACK_SIZE in exl_inline_hook_impl.s */
static constexpr int CTX_STACK_SIZE = 0x100;
/** Address of the inline hook entrypoint function */
extern "C" {
extern char exl_inline_hook_impl;
}

inline_always_ uintptr_t inline_hook_entrypoint() {
    return reinterpret_cast<uintptr_t>(&exl_inline_hook_impl);
}

static size_t s_count = 0;

/**
 * Allocate a new entry in the inline pool
 *
 * Panics if the pool is exhausted
 */
inline_always_ std::pair<const InlineHookEntry*, InlineHookEntry*>
allocate_inline_hook_entry() {
    if (s_count >= TOTAL) {
        panic_("Inline hook pool exhausted.");
    }
    auto ro_array =
        reinterpret_cast<const InlineHookEntry*>(s_inline_jit_pool.ro_start());
    auto rw_array =
        reinterpret_cast<InlineHookEntry*>(s_inline_jit_pool.rw_start());
    auto pair = std::make_pair(ro_array + s_count, rw_array + s_count);
    s_count++;
    return pair;
}

void install_inline_hook_at_offset(ptrdiff_t main_offset, uintptr_t callback) {
    install_inline_hook(megaton::module::main_info().start() + main_offset,
                        callback);
}

void install_inline_hook(uintptr_t hook, uintptr_t callback) {
    auto [entry_rx, entry_rw] = allocate_inline_hook_entry();

    /* Get pointer to entry's entrypoint. */
    uintptr_t trampoline_code =
        reinterpret_cast<uintptr_t>(&entry_rx->trampoline);

    /*
     * An inline hook is basically a trampoline hook, with the hook
     * implemented in assembly to backup and restore all registers
     */
    auto trampoline_ptr = install_trampoline_hook(hook, trampoline_code);
    // Offset of LR before SP is moved.
    static constexpr int lrBackupOffset =
        megaton::hook::InlineCtx::LR_OFFSET - CTX_STACK_SIZE;
    static_assert(lrBackupOffset == -0x10,
                  "ABI changed? - please fix InlineCtx");

    // Construct entrypoint instructions.
    entry_rw->trampoline = {
        /* Backup LR register to stack, as we are about to trash it. */
        exl::armv8::inst::SturUnscaledImmediate(
            exl::armv8::reg::LR, exl::armv8::reg::SP, lrBackupOffset),
        /* Branch to implementation. */
        exl::armv8::inst::BranchLink(
            inline_hook_entrypoint() -
            (trampoline_code + (1 * sizeof(exl::armv8::InstType)))),
        /* Restore proper LR. */
        exl::armv8::inst::LdurUnscaledImmediate(
            exl::armv8::reg::LR, exl::armv8::reg::SP, lrBackupOffset),
        /* Branch to trampoline. */
        exl::armv8::inst::Branch(
            trampoline_ptr -
            (trampoline_code + (3 * sizeof(exl::armv8::InstType))))};
    /* Assign callback to be called to be used by impl. */
    entry_rw->inline_hook_callback = callback;

    s_inline_jit_pool.flush();
}

} // namespace megaton::__priv
