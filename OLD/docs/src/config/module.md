# Build Config

The `Megaton.toml` file is used to configure the build options of your module.

## Module options
Open `Megaton.toml` in your project root. You should see:

```toml
[module]
name = "..."
title-id = 0xFFFFFFFFFFFFFFFF

[build]
sources = ["src"]
includes = ["src"]
```

`name` should already be auto-filled with the name of the folder when
you run `megaton init`. You need to replace `title-id` with the title ID
of the target game.

## Build options

### `libmegaton` and `entry`
By default, the build tool will link your module with `libmegaton`
and use its entry point `__megaton_module_entry`, which calls
your `megaton_main` function or your main function in Rust.

If you don't want to use `libmegaton`, you can set:
```toml
[build]
libmegaton = false
entry = "your_main_function"
```

`your_main_function` will be called by RTLD instead.

### `sources`
Sources are directories where C, C++, or assembly source files are located.
Nested directories will be automatically included.
```admonish info
Paths are relative to the project root (where `Megaton.toml` is located).
```

### `includes`
Include directories for C and C++ source files.

The default config assumes you will be putting headers and sources in the same
`src` directory (common for non-library projects). 
```admonish info
Paths are relative to the project root (where `Megaton.toml` is located).
```
```admonish tip
Using `-I` flags have the same effect. However, `includes` is preferred
because it lets your specify relative paths. The build tool will convert
the paths to absolute paths for the compiler.
```

### `libpaths` and `libraries`
These options are useful if you are linking with additional libraries.
`libpaths` is akin to `-L` flags, and `libraries` is akin to `-l` flags.
```toml
[build]
libpaths = ["path/to/lib"]
libraries = ["name"]
```

Like `includes`, paths are relative to the project root. Also,
the build tool will detect when libraries are updated to invoke the linker
again, even when nothing else has changed.

### `ldscripts`
This option is used to specify additional linker scripts. For example,
if you need to provide addresses to external functions
```toml
[build]
ldscripts = ["path/to/script.ld"]
```

## Build Flags
See [Build Flags](./build_flags.md) for configuring build flags.

## Cargo/Rust
This is not available yet
