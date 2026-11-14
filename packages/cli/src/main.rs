// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Contributors of https://github.com/Pistonite/megaton

mod cmd;
mod version;

#[cu::cli(preprocess=cmd::Cmd::preprocess)]
fn main(cmd: cmd::Cmd) -> cu::Result<()> {
    cmd.run()
}
