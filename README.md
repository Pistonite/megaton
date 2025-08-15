# megaton

NOTE: If you were using the megaton tool for build C projects,
please continue using the `old` branch while the new tool is being developed
(to support Rust, etc). ETA for the new tool is July 2026.

**In Development and VERY unstable**

Megaton is a build tool and support library for embedding Rust in a NSO binary. [(name reference)](https://www.zeldadungeon.net/wiki/Rusty_Switch)

## LICENSE
libmegaton is GPLv2. Everything else is MIT.

## Install
TODO

## Components
This project has the following components:
- `rustc`: Scripts to build rust with the targets we need
- TODO `runtime`: 
  - TODO Rust library and proc macros for setting up your rust app code
  - TOOD Basic implementation in C to get the NSO loaded by rtld
  - TODO absorb/rewrite `exlaunch`: A fork/modified version of [exlaunch](https://github.com/shadowninja108/exlaunch) that adds runtime patching and hooking support
  - TODO `hermit`: Proxy to forward hermit syscalls to NNSDK. This is a staticlib linked into the final ELF
- TODO `hammer`: CLI tool for building megaton projects


