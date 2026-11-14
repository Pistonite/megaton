// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

#[cu::cli]
fn main(_: cu::cli::Flags) -> cu::Result<()> {
    megaton_docker_setup::setup_isolated_crates("toolchain-image", &["toolchain", "config"])
}
