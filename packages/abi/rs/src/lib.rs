// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

#[unsafe(no_mangle)]
pub extern "C" fn sys_abort() {
    return;
}
 
#[cxx::bridge]
mod ffi {
    // #[namespace = "futex"] 
    // unsafe extern "C++" {
        // include!("../../cxx/include/futex.h");
        // unsafe fn sys_futex_wake(address: *mut u32, count: i32) -> i32;
    // }
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
