#!/bin/sh
echo '==> $ megaton --version'
megaton --version
echo '==> $ megaton toolchain check'
megaton toolchain check
echo '==> $ rustup toolchain list'
rustup toolchain list
echo '==> $ rustup +megaton target list --installed'
rustup +megaton target list --installed
