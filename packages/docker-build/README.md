# megaton-docker-build
Dockerfile for building a container that has devkitA64, Megaton and Megaton Rust toolchain.

The image also has Rust (stable) from building Megaton.
You don't need to setup Rust separately in CI unless you explicitly
need a different version.
