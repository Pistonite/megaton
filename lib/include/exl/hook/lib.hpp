#pragma once
#include <megaton/prelude.h>

#include <cstring>

#include <exl/hook/nx64/impl.hpp>
#include <exl/hook/nx64/inline_impl.hpp>
#include <exl/hook/util/func_ptrs.hpp>

namespace exl::hook {

/* TODO: 32-bit. */
namespace arch = nx64;

namespace {
using Entrypoint = util::GenericFuncPtr<void, void*, void*>;
};

inline void Initialize() {
    arch::Initialize();
    arch::InitializeInline();
}

template <typename InFunc, typename CbFunc>
CbFunc Hook(InFunc hook, CbFunc callback, bool do_trampoline = false) {

    /* Workaround for being unable to cast member functions. */
    /* Probably some horrible UB here? */
    uintptr_t hookp;
    uintptr_t callbackp;
    std::memcpy(&hookp, &hook, sizeof(hookp));
    std::memcpy(&callbackp, &callback, sizeof(callbackp));

    uintptr_t trampoline = arch::Hook(hookp, callbackp, do_trampoline);

    /* Workaround for being unable to cast member functions. */
    /* Probably some horrible UB here? */
    CbFunc ret;
    std::memcpy(&ret, &trampoline, sizeof(trampoline));

    return ret;
}

using InlineCtx = arch::InlineCtx;
using InlineCallback = void (*)(InlineCtx*);

inline void HookInline(uintptr_t hook, InlineCallback callback) {
    arch::HookInline(hook, reinterpret_cast<uintptr_t>(callback));
}
} // namespace exl::hook
