
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("lib.hpp");

        fn add(a: i32, b: i32) -> i32;
        fn hello(name: &str) -> String;
    }
}
