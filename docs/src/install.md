# Getting Started

Megaton is a build tool and library for linking and patching NX modules 
at runtime.

Credits:
- [exlaunch](https://github.com/shadowninja108/exlaunch) for the hooking and patching framework
  megaton is based on.

## Prerequisites
Make sure you have these installed:
- `git`
- Rust toolchain (`cargo`)

These are just for installing the tool. The tool itself requires
other tools to function, which you can install later.

```admonish warning
Currently, the installation is only tested on Linux.
```

## Install
1. First, clone the repository to where you want to install the tool
    ```bash
    git clone https://github.com/Pistonite/megaton path/to/install
    cd path/to/install
    ```
2. Build the tool
    ```bash
    cargo run --release --bin megaton-buildtool -- install
    ```
3. Add the `bin` directory to your `PATH`. For example, add the following to your shell profile:
    ```bash
    export PATH=$PATH:path/to/install/bin
    ```
4. Finish the installation
    ```bash
    megaton install
    ```

## Update
To update the tool in the future, you can run `install` with `-u`/`--update` flag:
```bash
megaton install -u
```
This will `git pull` the repository and rebuild the tool.

## Check Environment
To check that you have the necessary tools installed, run
```bash
megaton checkenv
```
This will check for additional tools required to build projects

```admonish warning
If any tool is missing, you will need to install it manually.
Rerun `megaton checkenv` after installing the tool to verify.
```

