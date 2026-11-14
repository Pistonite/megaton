// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

#[cu::cli]
async fn main(cmd: megaton_synctool::cmd::Cmd) -> cu::Result<()> {
    cmd.run_async(None).await
}
