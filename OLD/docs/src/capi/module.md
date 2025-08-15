# Module (C/C++)

## Module Information
The `<megaton/module.h>` header provides information about the module.

- Module name: `const char* megaton::module_name()`
- Length of module name: `size_t megaton::module_name_len()`
- Title ID: `u64 megaton::title_id()`
- Title ID as hex string: `const char* megaton::title_id_hex()`

## Module Layout
The `<megaton/module_layout.h>` header provides information about
runtime memory layout of the module

See [the source file](https://github.com/Pistonite/megaton/blob/main/lib/include/megaton/module_layout.h) for details
