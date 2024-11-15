#pragma once
#include <megaton/prelude.h>

#include <exl/hook/lib.hpp>
#include <exl/hook/macros.h>
#include <exl/hook/util/func_ptrs.hpp>

#define hook_trampoline_(name)                                                  \
    struct name : public ::exl::hook::impl::TrampolineHook<name>

namespace exl::hook::impl {

template <typename Derived> class TrampolineHook {

    template <typename T = Derived>
    using CallbackFuncPtr = decltype(&T::Callback);

    inline_always_ auto& OrigRef() {
        _HOOK_STATIC_CALLBACK_ASSERT();

        static constinit CallbackFuncPtr<> s_FnPtr = nullptr;

        return s_FnPtr;
    }

public:
    template <typename... Args>
    inline_always_ decltype(auto) Orig(Args&&... args) {
        _HOOK_STATIC_CALLBACK_ASSERT();

        return OrigRef()(std::forward<Args>(args)...);
    }

    inline_always_ void InstallAtOffset(ptrdiff_t address) {
        _HOOK_STATIC_CALLBACK_ASSERT();

        OrigRef() = hook::Hook(util::modules::GetTargetStart() + address,
                               Derived::Callback, true);
    }

    template <typename R, typename... A>
    inline_always_ void InstallAtFuncPtr(util::GenericFuncPtr<R, A...> ptr) {
        _HOOK_STATIC_CALLBACK_ASSERT();
        using ArgFuncPtr = decltype(ptr);

        static_assert(std::is_same_v<ArgFuncPtr, CallbackFuncPtr<>>,
                      "Argument pointer type must match callback type!");

        OrigRef() = hook::Hook(ptr, Derived::Callback, true);
    }

    inline_always_ void InstallAtPtr(uintptr_t ptr) {
        _HOOK_STATIC_CALLBACK_ASSERT();

        OrigRef() = hook::Hook(ptr, Derived::Callback, true);
    }
};

} // namespace exl::hook::impl
