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

==> $ megaton version -v
H] --- environment ---
D] MEGATON_HOME=/opt/megaton
D] cc: /opt/devkitpro/devkitA64/bin/aarch64-none-elf-gcc
D] cxx: /opt/devkitpro/devkitA64/bin/aarch64-none-elf-g++
D] as: /opt/devkitpro/devkitA64/bin/aarch64-none-elf-gcc
D] ar: /opt/devkitpro/devkitA64/bin/aarch64-none-elf-ar
D] objdump: /opt/devkitpro/devkitA64/bin/aarch64-none-elf-objdump
D] npdmtool: /opt/devkitpro/tools/bin/npdmtool
D] elf2nso: /opt/devkitpro/tools/bin/elf2nso
D] compiler version: 15.2.0
D] system header paths: [
 |     "/opt/devkitpro/devkitA64/aarch64-none-elf/include",
 |     "/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/15.2.0/aarch64-none-elf",
 |     "/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/15.2.0/backward",
 |     "/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/15.2.0",
 |     "/opt/devkitpro/devkitA64/lib/gcc/aarch64-none-elf/15.2.0/include",
 |     "/opt/devkitpro/devkitA64/lib/gcc/aarch64-none-elf/15.2.0/include-fixed",
 | ]
D] cxxbridge: /opt/megaton/bin/cxxbridge
H] --- toolchain ---
I] checking cxxbridge installation...
:: cxxbridge 1.0.197
:: location: /opt/megaton/bin/cxxbridge
I] checking megaton rust installation...
:: rustc 1.91.0-dev (caadc8df3 2025-08-14)
:: binary: rustc
:: commit-hash: caadc8df3519f1c92ef59ea816eb628345d9f52a
:: commit-date: 2025-08-14
:: host: x86_64-unknown-linux-gnu
:: release: 1.91.0-dev
:: LLVM version: 21.1.0
H] --- build tool ---
:: commit eb88e15f1270014ff66f000380a551e993638d83
:: version 0.1.2

==> $ rustup toolchain list
1.97.0-x86_64-unknown-linux-gnu (active, default)
megaton

==> $ rustup +megaton target list --installed
x86_64-unknown-linux-gnu
aarch64-unknown-hermit
aarch64-nintendo-switch-freestanding
```
