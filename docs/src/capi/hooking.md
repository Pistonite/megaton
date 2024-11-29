# Hooking in C++

The C++ Hooking API allows injecting custom callbacks into functions
in the main module.

There are 3 types of hooks:
- **Replace**: Replace the original function with your implementation
- **Trampoline**: Inject a trampoline callback in the original function. You
  can call the original function in the trampoline.
- **Inline**: Inject a trampoline callback at an instruction to inspect or
  modify the registers

To get started, include the `<megaton/hook.h>` header, then declare
a hook as a struct with provided macros. The following example
declares a hook to replace a function:

```cpp
#include <megaton/hook.h>

struct hook_replace_(foo) {
    target_offset_(0x00345678)
    static int call(int a, int b) {
        return a + b;
    }
};
```
Note:
- `hook_replace_` is macro for defining the hook struct.
  It can be replaced with `hook_trampoline_` or `hook_inline_` for a different hook type
- The `call` function defines the hook callback (i.e. code to run when the hooked function
  or code is called). It must be `static`
- For replace and trampoline hooks, the `call` function's signature should match
  the signature of the hooked function.
- The `target_offset_` macro declares the target offset in the main module
  where the hook will be installed. This is optional.

```admonish warning
When hooking a (non-static) member function, the callback should still
be a static function, with the `this` pointer passed as the first argument.
```

For inline hooks, the `call` function should take in an `InlineCtx` pointer,
which can be used to access the registers
```cpp
#include <megaton/hook.h>

struct hook_inline_(foobar) {
    target_offset_(0x00345678)
    static void call(InlineCtx* ctx) {
        // Your code here
        // read X20
        u64 x20 = ctx->x<20>();
        // set w8
        ctx->w<8>() = 0x12345678;
    }
};

```

## Installing the Hook
Once the hook struct is defined, it needs to be installed when your module
loads. Your main function's code path should at some point call one of the install
functions:

```cpp

// should be called by your main function
void install_my_hooks() {
    foo::install();
    foobar::install();
}

```

The `install` function is defined by the `target_offset_` macro, and
will install the function at an offset in the main module.

Without `target_offset_`, you can also manually specify the hook target
at install time:
```cpp
foobar::install_at_offset(0x00345678); // install at offset to main module
```
This is useful if the offset is not known at compile time.

## Pointer-to-member-function (PTMF) pitfalls
It's recommended to always use offsets for hooking, as function pointers
can have unexpected behavior with compilers. For example, the following
code will NOT work:

```cpp
struct hook_replace_(foo_hook) {
    static void call(void* this, int b) {
        // ...
    }
};

struct Foo {
    void foo(int b); // member function to hook, defined in the main module
};

void install_my_hooks() {
    foo_hook::install_at(reinterpret_cast<uintptr_t>(&Foo::foo));
}
```

This is because `Foo::foo` is a member function. Pointer-to-member-functions,
or PTMFs, have compiler-specific implementation and may not be a simple
(narrow) pointer. This is especially true for virtual functions.
