# megaton-docker-build
Dockerfile for building a container that has devkitA64, Megaton and Megaton Rust toolchain.

## Builder Image Info
```

==> $ /opt/devkitpro/devkitA64/bin/aarch64-none-elf-gcc --version
aarch64-none-elf-gcc (devkitA64) 15.2.0
Copyright (C) 2025 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

==> $ gcc --version
gcc (Debian 12.2.0-14+deb12u1) 12.2.0
Copyright (C) 2022 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

==> $ megaton --version
megaton v0.1.1 (c7409fff)

==> $ megaton toolchain check
I] checking cxxbridge installation...
:: cxxbridge 1.0.194
:: location: /opt/megaton/bin/cxxbridge
I] checking megaton rust installation...
:: rustc 1.91.0-dev (caadc8df3 2025-08-14)
:: binary: rustc
:: commit-hash: caadc8df3519f1c92ef59ea816eb628345d9f52a
:: commit-date: 2025-08-14
:: host: x86_64-unknown-linux-gnu
:: release: 1.91.0-dev
:: LLVM version: 21.1.0

==> $ rustup toolchain list
1.95.0-x86_64-unknown-linux-gnu (active, default)
megaton

==> $ rustup +megaton target list --installed
x86_64-unknown-linux-gnu
aarch64-unknown-hermit
aarch64-nintendo-switch-freestanding
```
