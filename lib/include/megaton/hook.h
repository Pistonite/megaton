/**
 * Runtime hooking system
 */
#pragma once
#include <megaton/prelude.h>
#include <utility>
#include <type_traits>

namespace megaton::__priv {

void init_hook();
void init_inline_hook();

uintptr_t do_install_hook(uintptr_t target, uintptr_t callback, bool is_trampoline);
uintptr_t do_install_hook_at_offset(ptrdiff_t target, uintptr_t callback, bool is_trampoline);
inline_always_ void install_replace_hook(uintptr_t target, uintptr_t callback) {
    do_install_hook(target, callback, false);
}
inline_always_ void install_replace_hook_at_offset(ptrdiff_t target, uintptr_t callback) {
    do_install_hook_at_offset(target, callback, false);
}
inline_always_ uintptr_t install_trampoline_hook(uintptr_t target, uintptr_t callback) {
    return do_install_hook(target, callback, true);
}
inline_always_ uintptr_t install_trampoline_hook_at_offset(ptrdiff_t target, uintptr_t callback) {
    return do_install_hook_at_offset(target, callback, true);
}

void install_inline_hook(uintptr_t target, uintptr_t callback);
void install_inline_hook_at_offset(ptrdiff_t main_offset, uintptr_t callback);

// Assertion to provide helpful diagnostics in IDE
#define assert_hook_callback_defined_ \
    static_assert(!std::is_member_function_pointer_v<decltype(&TDerived::call)>, "missing definition for `call` or missing `static` specifier for hook callback");
#define assert_derived_hook_callback_defined_ \
    static_assert(!std::is_member_function_pointer_v<decltype(&call)>, "missing definition for `call` or missing `static` specifier for hook callback");

/** Inline hook trait */
template<typename TDerived>
struct inline_hook {
    inline_member_ static void install_at(uintptr_t target) {
        assert_hook_callback_defined_;
        install_inline_hook(target, reinterpret_cast<uintptr_t>(TDerived::call));
    }
    inline_member_ static void install_at_offset(ptrdiff_t main_offset) {
        assert_hook_callback_defined_;
        install_inline_hook_at_offset(main_offset, reinterpret_cast<uintptr_t>(TDerived::call));
    }
};

/** Replace hook trait */
template<typename TDerived>
struct replace_hook {
    inline_member_ static void install_at(uintptr_t target) {
        assert_hook_callback_defined_;
        install_replace_hook(target, reinterpret_cast<uintptr_t>(TDerived::call));
    }
    inline_member_ static void install_at_offset(ptrdiff_t main_offset) {
        assert_hook_callback_defined_;
        install_replace_hook_at_offset(main_offset, reinterpret_cast<uintptr_t>(TDerived::call));
    }
};

/** Trampoline hook trait */
template<typename TDerived>
struct trampoline_hook {
    inline_member_ static auto& trampoline() {
        static constinit decltype(&TDerived::call) _trampoline = nullptr;
        return _trampoline;
    }
    inline_member_ static void install_at(uintptr_t target) {
        assert_hook_callback_defined_;
        trampoline() = reinterpret_cast<decltype(&TDerived::call)>(install_trampoline_hook(
            target, 
            reinterpret_cast<uintptr_t>(TDerived::call)
        ));
    }
    inline_member_ static void install_at_offset(ptrdiff_t main_offset) {
        assert_hook_callback_defined_;
        trampoline() = reinterpret_cast<decltype(&TDerived::call)>(install_trampoline_hook_at_offset(
            main_offset, 
            reinterpret_cast<uintptr_t>(TDerived::call)
        ));
    }
    template <typename... Args>
    inline_always_ decltype(auto) call_original(Args&&... args) {
        assert_hook_callback_defined_;
        return trampoline()(std::forward<Args>(args)...);
    }
};

}

#define hook_inline_(name) name: public ::megaton::__priv::inline_hook<name>
#define hook_replace_(name) name: public ::megaton::__priv::replace_hook<name>
#define hook_trampoline_(name) name: public ::megaton::__priv::trampoline_hook<name>

#define target_offset_(main_offset) \
    static constexpr ptrdiff_t s_offset = main_offset; \
    inline_always_ void install(void) { \
        assert_derived_hook_callback_defined_; \
        static bool installed = false; \
        if (installed) { \
            return; \
        } \
        installed = true; \
        install_at_offset(s_offset); \
    }

#define target_(target) \
    static constexpr uintptr_t s_target = target; \
    inline_always_ void install(void) { \
        assert_derived_hook_callback_defined_; \
        static bool installed = false; \
        if (installed) { \
            return; \
        } \
        installed = true; \
        install_at(s_target); \
    }

namespace megaton::hook {

/**
 * Initialize the hooking system
 *
 * This is automatically called in the megaton entrypoint, before
 * user-defined megaton_main
 */
inline_always_ void init() {
    megaton::__priv::init_hook();
    megaton::__priv::init_inline_hook();
}

/** Access register value */
union Reg {
    /** 64-bit register */
    u64 x;
    /** 32-bit register */
    u32 w;
};

/**
 * Register context for inline hooks
 */
struct InlineCtx {
    Reg reg[31];

    static constexpr int LR_OFFSET = 0xf0;

    /** Access the frame pointer */
    inline_member_ u64& fp() { return reg[29].x; }
    /** Access the link register */
    inline_member_ u64& lr() { return reg[30].x; }
    /** Access the frame pointer */
    inline_member_ const u64& fp() const { return reg[29].x; }
    /** Access the link register */
    inline_member_ const u64& lr() const { return reg[30].x; }
    /** Access 64-bit general-purpose registers */
    template <u8 index>
    inline_member_ u64& x() { 
        static_assert(index < 31, "register index out of bounds");
        return reg[index].x;
    }
    /** Access 64-bit general-purpose registers */
    template <u8 index>
    inline_member_ const u64& x() const { 
        static_assert(index < 31, "register index out of bounds");
        return reg[index].x;
    }
    /** Access 32-bit general-purpose registers */
    template <u8 index>
    inline_member_ u32& w() { 
        static_assert(index < 31, "register index out of bounds");
        return reg[index].w;
    }
    /** Access 32-bit general-purpose registers */
    template <u8 index>
    inline_member_ const u32& w() const { 
        static_assert(index < 31, "register index out of bounds");
        return reg[index].w;
    }
};

}
