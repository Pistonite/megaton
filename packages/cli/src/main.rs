// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

use megaton_cmd::cmds;

#[cu::cli(preprocess=cmds::CmdMegaton::preprocess)]
fn main(cmd: cmds::CmdMegaton) -> cu::Result<()> {
    cmds::main(cmd)
}
