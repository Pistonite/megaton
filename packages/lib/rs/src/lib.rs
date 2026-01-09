// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

pub use abi::sys_abort;
// mod ffi {
//
//     // essentially becomes "visible" to foreign code
//
//
//     #[namespace = "example"]
//     // ^ this means everything inside is in namespace example { ... }
//     unsafe extern "C++" {
//         // this includes include/example-mod/my_mod.h
//         // the include path is defined in the build script
//         include!("example-mod/my_mod.h");
//
//         // declaring it here makes the function "visible" to Rust
//         // as you can see, this function is also declared in my_mod.h
//         fn init_function_in_c();
//     }
//
//     #[namespace = "example_rs"]
//     // ^ this means everything inside will be generated as namespace example_rs { ... }
//     extern "Rust" {
//         // declaring the function here makes the function "visible" to C++
//         fn get_int_from_rust(input: i32) -> i32;
//     }
// }
//
// // this is just a normal Rust function, correspond
// // to the "declaration" we put above
// fn get_int_from_rust(_input: i32) -> i32 {
//     panic!();
//
//     //input + 42
// }
//
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
