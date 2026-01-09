// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
