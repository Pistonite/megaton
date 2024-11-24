#pragma once
#include <megaton/module_layout.h>

#include <exl/hook/lib.hpp>
#include <exl/hook/macros.h>
#include <exl/hook/util/func_ptrs.hpp>

#define hook_replace_(name)                                                    \
    struct name : public ::exl::hook::impl::ReplaceHook<name>

namespace exl::hook::impl {

template <typename Derived> struct ReplaceHook {

    template <typename T = Derived>
    using CallbackFuncPtr = decltype(&T::Callback);

    inline_always_ void InstallAtOffset(ptrdiff_t address) {
        _HOOK_STATIC_CALLBACK_ASSERT();

        hook::Hook(megaton::module::main_info().start() + address,
                   Derived::Callback);
    }

    template <typename R, typename... A>
    inline_always_ void InstallAtFuncPtr(util::GenericFuncPtr<R, A...> ptr) {
        _HOOK_STATIC_CALLBACK_ASSERT();

        using ArgFuncPtr = decltype(ptr);
        static_assert(std::is_same_v<ArgFuncPtr, CallbackFuncPtr<>>,
                      "Argument pointer type must match callback type!");

        hook::Hook(ptr, Derived::Callback);
    }

    inline_always_ void InstallAtPtr(uintptr_t ptr) {
        _HOOK_STATIC_CALLBACK_ASSERT();

        hook::Hook(ptr, Derived::Callback);
    }
};

} // namespace exl::hook::impl
