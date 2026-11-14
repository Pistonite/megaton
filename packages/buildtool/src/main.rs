// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

#[cu::cli]
fn main(cmd: megaton_buildtool::cmd::Cmd) -> cu::Result<()> {
    cmd.run(None)
}
