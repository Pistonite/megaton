// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

#[unsafe(no_mangle)]
pub extern "C" fn sys_abort() {
    return;
}
 
// #[cxx::bridge]
// mod ffi {
//     #[namespace = "futex"] 
//     // ^ this means everything inside is in namespace futex { ... }
//     unsafe extern "C++" {
//         // this includes include/example-mod/my_mod.h
//         // the include path is defined in the build script
//         include!("../cxx/include/futex.h");

//         // declaring it here makes the function "visible" to Rust
//         // as you can see, this function is also declared in my_mod.h
//         unsafe fn futex_wake_impl(address: *mut u32, count: i32) -> i32;
//     }
// }

/*
Questions:

- What am I able to call?
    - Can I call libnx stuff from Rust?
- How can I make nx syscalls?
- 

*/


// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn sys_futex_wake(address: *mut u32, count: i32) -> i32 {
//     unsafe {
//         return crate::ffi::futex_wake_impl(address, count);

//     }
// }


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
