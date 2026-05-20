# Build Flags

Megaton uses a preset list of flags for the build toolchain both on the rust and
C/C++ sides. These flags should handle targeting the switch architecture and
platform. Certain optomizations are also enabled and certain features are diabled.
It is recommended to use all of the default flags since Megaton has not been tested
with any of the build flags disabled. However, you may add flags to certain parts
of the build pipeline, e.g., to add `-D` define statements or enable certain behavior
for your target game. These flags are all set in the `build.flags` section of the
Megaton config file. Certain command line flags, such as specifing a linker script,
are included implicitly and should be specified in the appropriate `build` config
option. See the full [build config](../reference/configuration/section_build.html)
documentation for a full description of how default flags are combined to form the
build commands.

```toml
[build.flags]
c = ["<default>", "-DMY_DEFINE=1"]
rust = ["<default>", "-Zub-checks=yes"]
cargo = ["<default>", "--features", "my-feature"]
```

There are 7 sections for flags to be added to, corresponding to different compilers
or parts of the build toolchain.

## Common
These flags are applied to all non-rust toolchain components, i.e. `cc`, `cxx`, 
`as`, and `ld`.

#### Defaults
```
-march=armv8-a+crc+crypto
-mtune=cortex-a57
-mtp=soft
-fPIC
-fvisibility=hidden
-g
```

## C
These flags are passed to the C compiler when compiling C sources.

#### Defaults
```
-Wall
-Werror
-ffunction-sections
-fdata-sections
-O3
```

## CXX
These flags are passed to the C++ compiler when compiling C++ sources.

#### Defaults
```
-std=c++20
-fno-rtti
-fno-exceptions
-fno-asynchronous-unwind-tables
-fno-unwind-tables
```

## AS
These flags are passed to the assembler when compiling assembly sources.
There are currently no default flags used specifically for the assembler.
Note that the assembler tool used is actually gcc, so flags should be
formatted as gcc flags when used as an assembler.

## LD
These flags are passed to the linker when linking the ELF from all compiled
binary objects. Again, these are formatted as link flags for g++, not 
for ld itself.

#### Defaults
```
-nostartfiles
-nodefaultlibs
-Wl,--shared
-Wl,--export-dynamic
-Wl,-z,nodynamic-undefined-weak
-Wl,--build-id=sha1
-Wl,--gc-sections
-Wl,--nx-module-name
```

## Rust
Flags passed to the rust compiler. Equivalent to the RUSTFLAGS environment variable.
Place flags here instead of Cargo.toml if you want them to be profile
controlled.

Currently no default rust flags.

## Cargo
Flags passed to cargo for building all rust code. The `+megaton` toolchain is
also implicitly included.

#### Defaults
```
--release
--target aarch64-unknown-hermit
```
