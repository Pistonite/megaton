## Module Name
Module name is defined in Megaton.toml

translated to `-DMEGATON_NX_MODULE_NAME=<value>` by build tool
and `-DMEGATON_NX_MODULE_NAME_LEN=<value>`
and environment variable `MEGATON_NX_MODULE_NAME` for cargo (Rust side)

`nx-module-name` is special linker section, and the `--nx-module-name`
linker flag sets it

bootstrap C provides:
- `__megaton_module_name()`
- `__megaton_module_name_len()`

lib exposes:
- `megaton::module_name()`
- `megaton::module_name_len()`


On Rust side, bootstrap macro will include
```rust
#[no_mangle]
fn __megaton_module_name_rs() -> &'static str {
    env!("MEGATON_NX_MODULE_NAME")
}
```

librs should have
```rust
extern "Rust" {
    fn __megaton_module_name_rs() -> &'static str;
}
pub mod megaton {
    pub fn module_name() -> &'static str {
        __megaton_module_name_rs()
    }
}
```

## Init
init sequence is
- entry
- megaton C init
- megaton rs init through FFI
  - rs init function provided by rs bootstrap
- userland C init, which bootstraps to userland rust init

If rust support is enabled, `-DMEGATON_RUST` is defined

```c

extern "C" {
// real entry point of the module
void __megaton_module_entry() {
    __megaton_module_init();
#ifdef MEGATON_RUST
    __megaton_rs_module_init();
#endif
    megaton_main();
}
}

extern "C" void megaton_main() {
    // init rust side through megaton
    megaton::rs_main();

    // user init
}

#define megaton_main_

megaton_main_() {
}
```

## Panic and Abort Stuff

C side:
lib will provide these APIS:
- __megaton_handle_panic(file, line, msg)
  - formats panic message
  - call hooks
  - abort
- __megaton_add_panic_hook(callback)
  - adds a callback to be called when a panic occurs, with the message
  - callback can log the message as needed however it wants

The following macros are provided
- `panic_(msg)` - uses `__FILE__` and `__LINE__` to call `__megaton_handle_panic`
- `unreachable_()` - calls `panic_` with "unreachable"
- `assert_(condition)` - calls `panic_` with "assertion failed" and a condition

Rust side:
librs will have a panic hook that 
formats the panic message, and call `__megaton_handle_panic` through FFI

The usual `panic!`, `unreachable!`, `assert!` macros will work,
and `.rs` files and lines will be included in the panic system.

## nx-module-name
