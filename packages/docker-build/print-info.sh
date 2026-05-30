#!/bin/sh
echo '==> $ /opt/devkitpro/devkitA64/bin/aarch64-none-elf-gcc --version'
/opt/devkitpro/devkitA64/bin/aarch64-none-elf-gcc --version
echo '==> $ gcc --version'
gcc --version
echo '==> $ megaton version -v'
megaton version -v
echo ''
echo '==> $ rustup toolchain list'
rustup toolchain list
echo ''
echo '==> $ rustup +megaton target list --installed'
rustup +megaton target list --installed
