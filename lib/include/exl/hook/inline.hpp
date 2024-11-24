#pragma once
#include <megaton/module_layout.h>

#include <exl/hook/lib.hpp>
#include <exl/hook/macros.h>

#define hook_inline_(name)                                                     \
    struct name : public ::exl::hook::impl::InlineHook<name>

namespace exl::hook::impl {

template <typename Derived> struct InlineHook {

    template <typename T = Derived>
    using CallbackFuncPtr = decltype(&T::Callback);

    inline_always_ void InstallAtOffset(ptrdiff_t address) {
        _HOOK_STATIC_CALLBACK_ASSERT();

        hook::HookInline(megaton::module::main_info().start() + address,
                         Derived::Callback);
    }

    inline_always_ void InstallAtPtr(uintptr_t ptr) {
        _HOOK_STATIC_CALLBACK_ASSERT();

        hook::HookInline(ptr, Derived::Callback);
    }
};
} // namespace exl::hook::impl
