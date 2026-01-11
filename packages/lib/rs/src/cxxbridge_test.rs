#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        fn add_numbers(a: i32, b: i32) -> i32;
    }
}

