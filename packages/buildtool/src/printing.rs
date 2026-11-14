// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Contributors of https://github.com/Pistonite/megaton

pub async fn join_stdstream(handle: cu::co::Handle<cu::Result<Vec<u8>>>) -> Option<String> {
    let Ok(bytes) = handle.co_join().await.flatten() else {
        return None;
    };
    let msg = String::from_utf8_lossy(&bytes);
    let msg = msg.trim();
    if msg.is_empty() {
        return None;
    }
    Some(msg.to_string())
}
