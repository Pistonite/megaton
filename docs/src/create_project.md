# Create Project

To initialize a new project, run the following command in an empty directory:
```bash
megaton init
```

This generates the following structure:
```
- src/
  - main.cpp
- Megaton.toml
- .clang-format
- .clangd
- .gitignore
```

## Build
To build the project the first time, you need to add the `--lib` flag
to also build `libmegaton`. You don't need to do this for future builds
of this and other projects, unless you updated the tool and need to rebuild
the library
```admonish info
Most of the time you will want to use the megaton library.
However if you don't, you can set `build.libmegaton` to `false`
in the config, and skip it here
```

```bash
megaton build --lib
```

This should put the output NSO at `target/megaton/none/<name>.nso`

```admonish tip
The build command also generates a `clangd`-compatible compile DB (`compile_commands.json`).
The generated `.clangd` config already references this file, along
with other useful settings to make it work out of the box for you
```

## Entrypoint
Open `src/main.cpp` and you will see the following:
```cpp
#include <megaton/prelude.h>

extern "C" void megaton_main() {
    // Your code here
}
```

The `megaton/prelude.h` include file includes primitive type definitions
like `i32` and `u32` as well as `panic_` macros to help interfacing
with the panic system. `megaton_main` is the main function of your module.

```admonish info
Libmegaton provides an entry point `__megaton_module_entry` that initializes
the library and calls your main function. The entry point is called by
RTLD when the module is loaded.
```
