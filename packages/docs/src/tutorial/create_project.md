# Create Project

> [!NOTE]
> At the moment, there is no `megaton init` command, so the initial files
> will need to be created manually. When this command is added in the future,
> this step will be much faster and easier.
>
> <https://github.com/Pistonite/megaton/issues/136>

## Create Megaton Config
To initialize a Megaton project, you will need a `Megaton.toml` config file.
Add this file in an empty directory, which will be the root directory of the
Megaton project.

```toml
# Megaton.toml

# The version of Megaton to use.
# If the version of Megaton operating on this config file does not
# satisfy the version, it will error.
megaton.version = "^0.3.0"

[module]
# The name of the compiled module. This will be the name of the output NSO
name = "my-mod"
# If you want to have versioning for your mod, you can use this field.
# Megaton ignores this completely
version = "0.1.0"

# Change it to your game's title id
title-id = 0x0
# Megaton has a sync tool that will upload the mod via filesystem (USB)
# or FTP. This is the target file name for the NSO.
nso-name = "subsdk9"

# Options for building C/C++ sources
[build]
# Source directories. C/C++/assembly files inside are recursively scanned
sources = ["src"]
# Include directories.
includes = ["include"]
```
See the pages under [Configuration](../reference/configuration/index.html)
for a full specification of all the config options.

## Create Cargo Project

If you are using rust, initialize a cargo project. Megaton will automatically
enable Rust support if it finds a `Cargo.toml` in the same directory.
You can also configure it to look in another place.

```toml
# Cargo.toml

[package]
# Currently the crate name is used as the import namespace for CXX-generated
# headers. In your C++ code, you will include Rust files like `#include <my-mod-rs/lib.rs.h>`
name = "my-mod-rs"
version = "0.1.0"
edition = "2024"

[dependencies]
# The megaton Rust library needs to be added for utilities that allow
# the Rust side to talk to the C++ side
megaton.path = "target/megaton/lib"

# Megaton has built-in support for the `cxx` crate for Rust-CXX interop.
# The upstream crate ships breaking changes pretty often, so Megaton
# typically has a "blessed" version that is locked with each release.
# You can run `megaton -vV` to check the "blessed" version.
#
# If you are using a higher version, it will print a warning. If you are using
# a lower version, it will return an error prompting you to upgrade.
cxx = "1.0.197"

[lib]
# Crate must be a static library in order for Megaton to process the output
crate-type = ["staticlib"]
```

```rust
// src/lib.rs

// Put a `megaton::main` attribute on the initialization function of the Rust
// side of your mod. This will generate shims around the unsafe ABI and generate
// code to initialize the Rust side of Megaton's library.
//
// You can call this function anything
#[megaton::main]
fn my_mod_rust_main() {
    
}
```

## Add initial C++ Code

Finally, you need to define `megaton_main` in the C++ side, which is the entry point
of the whole mod.

```cpp
// src/main.cpp

// TODO: figure out what header to put megaton::rust_main
#include <megaton/what?.h>

// This is the main mod entry point which will be called at runtime
// when you mod initializes
extern "C" void megaton_main() {
    /* do your mod stuff here */

    // NOTE: If you use Rust, you must manually call megaton::rust_main(),
    // which starts the initialization of the Rust module.
    // This gives you full control over the initialization sequence.
    megaton::rust_main();
}
```

Now run ```bash`megaton build``` and the mod should build the NSO to `target/megaton/none/my-mod/my-mod.nso`

> [!NOTE]
> Like most modern build systems such as Cargo, Megaton is able to automatically
> determine what to build based on the config file and incremental-build information
> from the compiler. Unlike a traditional C++ project with CMake, you will typically
> not need a separate "configure" step.
>
> This also means in cases where a CMake project will require a manual re-configure,
> such as adding a new file or source directory, Megaton-build will do that automatically.
> It's considered a bug if you have to clean the output directory to produce a proper build.
