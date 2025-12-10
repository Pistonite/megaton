#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn add_numbers(a: i32, b: i32) -> i32;
    }
}

fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
