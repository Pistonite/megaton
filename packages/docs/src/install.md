# Installation

## Prerequisites
- Megaton requires `DevKitA64` from [DevKitPro](https://devkitpro.org/wiki/Getting_Started).
  (Select the `switch-dev` group)

  > [!NOTE]
  > Please set the `DEVKITPRO` environment variable for Megaton to be able to
  > find the DKP installation:
  >
  > For example:
  > ```bash
  > DEVKITPRO=/opt/devkitpro
  > ```

- For building mods written in Rust, build tools are required to build the Rust
  toolchain:
    - Git
    - CMake
    - Ninja
    - A C/CXX Toolchain
    - A Rust Toolchain on the host system (for running `cargo`).

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
this toolchain at ```bash`$MEGATON_HOME/rust-toolchain```. ```bash`$MEGATON_HOME```
defaults to `~/.cache/megaton`.

This toolchain is locked at a commit for every Megaton version. If you need Rust feature in a higher version of Rust than
what is included with Megaton, please open an issue on GitHub.

Run the following command to install the toolchain, or upgrade the toolchain in the future
```bash
megaton toolchain install
```

> [!NOTE]
> This compiles LLVM and bootstraps the Rust compiler, which will take a while.
> To keep the build artifact, use the `-k`/`--keep` flag: ```bash`megaton toolchain install -k```.
> This will make it faster to rebuild the toolchain in the future when upgrading. However,
> this will consume 10-20GB of disk spaces.

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

