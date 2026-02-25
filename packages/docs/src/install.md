# Installation

## Prerequisites
- Megaton requires `DevKitA64` from [DevKitPro](https://devkitpro.org/wiki/Getting_Started).
  (Select the `switch-dev` group)
- For Rust support (building mods written in Rust), a Rust toolchain for the host system
  (i.e. the system used to build the mod) and `git` are required to clone and build
  the Rust compiler for the `aarch64-unknown-hermit` target.

## Install Megaton
The `megaton` CLI is a single binary, which can be installed in 3 ways:
1. (Recommended) Install from prebuilt binary with `cargo-binstall`
    ```bash
    cargo binstall megaton-cmd --git https://github.com/Pistonite/megaton
    ```
2. Install from source with `cargo`
    ```bash
    cargo install megaton-cmd --git https://github.com/Pistonite/megaton
    ```
3. Download the binary from the latest release on [GitHub](https://github.com/Pistonite/megaton/releases)

## Install Megaton Rust Toolchain
Megaton implements the [Hermit ABI](https://github.com/hermit-os/hermit-rs) to bind Rust
Standard Library to NNSDK. This requires a custom Rust toolchain. Megaton will install
this toolchain at `~/.cache/megaton/rust-toolchain`

Run the following command to install the toolchain, or upgrade the toolchain in the future
```bash
megaton toolchain install
```

```admonish note
This compiles LLVM and bootstraps the Rust compiler, which will take a while.
To keep the build artifact, use the `-k`/`--keep` flag: `megaton toolchain install -k`.
This will make it faster to rebuild the toolchain in the future when upgrading. However,
this will consume 10-20GB of disk spaces.
```

You can check if the toolchain is installed with
```bash
megaton toolchain check
```
Or directly with `rustc` (`megaton` is the name of the toolchain)
```bash
rustc +megaton -vV
```

## Upgrading
To upgrade `megaton`, simply reinstall the latest version with `cargo`/`cargo-binstall`,
or replace the binary with the latest release on GitHub if you installed it manually.

To upgrade the custom Rust toolchain, run `megaton toolchain install` after upgrading `megaton`.

