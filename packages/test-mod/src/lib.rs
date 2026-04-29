// use std::io::prelude::*;
// use std::net::TcpStream;
// look at this file and include/example-mod/my_mod.h at the same time

use crate::ffi::write_test_output;

// here, we are using the cxx crate to do Rust-C++ interop
// see https://cxx.rs/index.html
#[cxx::bridge]
mod ffi {
    // anything declared inside #[cxx::bridge]
    // essentially becomes "visible" to foreign code

    #[namespace = "example"]
    // ^ this means everything inside is in namespace example { ... }
    unsafe extern "C++" {
        // this includes include/example-mod/my_mod.h
        // the include path is defined in the build script
        include!("new-example-mod/my_mod.h");

        // declaring it here makes the function "visible" to Rust
        // as you can see, this function is also declared in my_mod.h
        fn init_function_in_c();
        fn write_test_output(data: &str);
    }
    #[namespace = "example_rs"]
    // ^ this means everything inside will be generated as namespace example_rs { ... }
    extern "Rust" {
        // declaring the function here makes the function "visible" to C++
        fn run_megaton_tests();
    }
}

// unsafe extern "C" {
//     #[link_name = "_ZN4botw3tcp5sendfEPKcz"]
//     unsafe fn sendf(format: *const std::ffi::c_char, ...);
// }

static mut total_tests: usize = 0;
static mut passed_tests: usize = 0;

fn megaton_log(text: &str) {
    // send over tcp
    // let cs = std::ffi::CString::new(text).unwrap();
    // unsafe { sendf(cs.as_ptr()); }
    // TODO: save to filesystem
    ffi::write_test_output(text);
    // TODO: any additional logging
}

fn megaton_assert<T: std::cmp::PartialEq + std::fmt::Debug>(result: T, expected: T) -> bool {
    let test_num: usize;
    unsafe {
        total_tests += 1;
        test_num = total_tests;
    }
    if result != expected {
        megaton_log(format!("Test number {test_num} failed: got {:#?}, expected {:#?}\n", result, expected).as_str());
        return false;
    } else {
        unsafe { passed_tests += 1 };
        return true;
    }
}

fn run_megaton_tests() {
    let test_total: usize;
    let tests_passed: usize;
    unsafe {
        test_total = total_tests;
        tests_passed = passed_tests;
    }
    megaton_log("tests have run\n");
    megaton_log(format!("{tests_passed}/{test_total} Passed\n").as_str());
}

// this shows raw FFI without using the cxx crate
// `no_mangle` makes the symbol exported from the final
// staticlib as-is.

// we want to eventualy hide all the no_mangle and unsafe stuff
// from the user, since those are implementation details.
// we can do this through a proc-macro, something like:
// ```rust
// #[megaton::main]
// fn main() {
//     ffi::init_function_in_c
// }
// ```
//
// The #[megaton::main] attribute is a macro that
// generates (some code equivalent to) the code below:

#[unsafe(no_mangle)]
extern "C" fn __megaton_rs_main() {
    // here just as an example, we call back to C code
    // to actually do the init,
    // but imagine the user might want to do some rust-side
    // initialization
    ffi::init_function_in_c();
}

// Now, keep this file open, close example-mod/my_mod.h, and open src/main.cpp
