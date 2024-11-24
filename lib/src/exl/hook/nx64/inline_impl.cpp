#include <megaton/prelude.h>

#include <array>

#include <exl/hook/nx64/impl.hpp>
#include <exl/hook/nx64/inline_impl.hpp>
#include <exl_armv8/prelude.h>

#include <megaton/__priv/jit.h>
#include <megaton/align.h>

/** Size of inline JIT memory. */
#ifndef MEGATON_INLINE_JIT_SIZE
#define MEGATON_INLINE_JIT_SIZE 0x1000
#endif

namespace exl::hook::nx64 {

/* Size of stack to reserve for the context. Adjust this along with
 * CTX_STACK_SIZE in inline_asm.s */
static constexpr int CtxStackSize = 0x100;

static constexpr usize InlinePoolSize = MEGATON_INLINE_JIT_SIZE;
static_assert(align_up_(InlinePoolSize, PAGE_SIZE) == InlinePoolSize,
              "InlinePoolSize is not aligned");

namespace reg = exl::armv8::reg;
namespace inst = exl::armv8::inst;

struct Entry {
    std::array<inst::Insn, 4> m_CbEntry;
    uintptr_t m_Callback;
};

static constexpr size_t InlinePoolCount =
    MEGATON_INLINE_JIT_SIZE / sizeof(Entry);

make_jit_(s_InlineHookJit, MEGATON_INLINE_JIT_SIZE);

static size_t s_EntryIndex = 0;

extern "C" {
extern char exl_inline_hook_impl;
}

static uintptr_t GetImpl() {
    return reinterpret_cast<uintptr_t>(&exl_inline_hook_impl);
}

static const Entry* GetEntryRx() {
    return reinterpret_cast<const Entry*>(s_InlineHookJit.ro_start());
}

static Entry* GetEntryRw() {
    return reinterpret_cast<Entry*>(s_InlineHookJit.rw_start());
}

void InitializeInline() { s_InlineHookJit.init(); }

void HookInline(uintptr_t hook, uintptr_t callback) {
    /* Ensure enough space in the pool. */
    if (s_EntryIndex >= InlinePoolCount) {
        panic_("Inline hook pool exhausted.");
    }

    /* Grab entry from pool. */
    auto entryRx = &GetEntryRx()[s_EntryIndex];
    auto entryRw = &GetEntryRw()[s_EntryIndex];
    s_EntryIndex++;

    /* Get pointer to entry's entrypoint. */
    uintptr_t entryCb = reinterpret_cast<uintptr_t>(&entryRx->m_CbEntry);
    /* Hook to call into the entry's entrypoint. Assign trampoline to be used by
     * impl. */
    auto trampoline = Hook(hook, entryCb, true);
    /* Offset of LR before SP is moved. */
    static constexpr int lrBackupOffset =
        int(offsetof(InlineCtx, m_Gpr.m_Lr)) - CtxStackSize;
    static_assert(lrBackupOffset == -0x10, "InlineCtx is not ABI compatible.");

    /* Construct entrypoint instructions. */
    auto impl = GetImpl();
    entryRw->m_CbEntry = {
        /* Backup LR register to stack, as we are about to trash it. */
        inst::SturUnscaledImmediate(reg::LR, reg::SP, lrBackupOffset),
        /* Branch to implementation. */
        inst::BranchLink(impl - (entryCb + (1 * sizeof(armv8::InstType)))),
        /* Restore proper LR. */
        inst::LdurUnscaledImmediate(reg::LR, reg::SP, lrBackupOffset),
        /* Branch to trampoline. */
        inst::Branch(trampoline - (entryCb + (3 * sizeof(armv8::InstType))))};
    /* Assign callback to be called to be used by impl. */
    entryRw->m_Callback = callback;

    /* Finally, flush caches to have RX region to be consistent. */
    s_InlineHookJit.flush();
}
} // namespace exl::hook::nx64
