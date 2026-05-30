// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use megaton_cmd::cmds::Cmd;

#[cu::cli(preprocess=Cmd::preprocess)]
fn main(cmd: Cmd) -> cu::Result<()> {
    cmd.run()
}
