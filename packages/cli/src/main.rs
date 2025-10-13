// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

use megaton_cmd::cmds;

#[cu::cli]
fn main(cmd: cmds::CmdMegaton) -> cu::Result<()> {
    cmds::main(cmd)
}
