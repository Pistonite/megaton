// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

mod fs_tests;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("test-mod/mod.h");

        fn init_function_in_c();
        fn write_test_output(data: &str);
    }
    #[namespace = "example_rs"]
    extern "Rust" {
        fn run_megaton_tests();
    }
}

struct MegatonTests {
    total_tests: usize,
    passed_tests: usize,
    category_tests: usize,
    category_passed_tests: usize,
    category: &'static str
}

impl MegatonTests {

    fn new() -> MegatonTests {
        MegatonTests {
            total_tests: 0,
            passed_tests: 0,
            category_tests: 0,
            category_passed_tests: 0,
            category: "",
        }
    }

    fn megaton_log(&self, text: &str) {
        ffi::write_test_output(text);
    }

    fn megaton_assert<T: std::cmp::PartialEq + std::fmt::Debug>(&mut self, result: T, expected: T) {
        self.total_tests += 1;
        self.category_tests += 1;
        if result != expected {
            self.megaton_log(format!("Test number {:#?} failed: got {:#?}, expected {:#?}\n", self.total_tests, result, expected).as_str());
        } else {
            self.passed_tests += 1;
            self.category_passed_tests += 1;
        }
    }

    fn megaton_assert_msg<T: std::cmp::PartialEq + std::fmt::Debug>(&mut self, result: T, expected: T, msg: &str) {
        self.total_tests += 1;
        self.category_tests += 1;
        if result != expected {
            self.megaton_log(format!("Test number {:#?} failed: got {:#?}, expected {:#?}. Message: {:?}\n", self.total_tests, result, expected, msg).as_str());
        } else {
            self.passed_tests += 1;
            self.category_passed_tests += 1;
        }
    }

    fn megaton_assert_ok<T,E>(&mut self, result: Result<T,E>, msg: &str) -> Option<T> 
    where 
        T: std::fmt::Debug,
        E: std::fmt::Debug 
        {
        self.total_tests += 1;
        self.category_tests += 1;
        if result.is_err() {
            self.megaton_log(format!("Test number {:#?} failed: received Err: {:?}. Message: {:?}\n", self.total_tests, result.unwrap_err(), msg).as_str());
            return None;
        } else {
            self.passed_tests += 1;
            self.category_passed_tests += 1;
            return Some(result.unwrap());
        }
    }
    fn start_category(&mut self, category: &'static str) {
        self.category_tests = 0;
        self.category_passed_tests = 0;
        self.category = category;
    }

    fn end_category(&mut self) {
        self.megaton_log(format!("{:#?} tests finished, {:#?}/{:#?} Passed\n", self.category, self.category_passed_tests, self.category_tests).as_str());
        self.category_tests = 0;
        self.category_passed_tests = 0;
        self.category = "";
    }
}

fn megaton_num_tests(mtt: &mut MegatonTests) {
    mtt.start_category("Math");
    // basic math
    mtt.megaton_assert(1 + 1, 2);
    mtt.megaton_assert(1 * 1, 1);
    mtt.megaton_assert(1 - 1, 0);
    mtt.megaton_assert(1 / 1, 1);
    mtt.megaton_assert(4 + 2, 6);
    mtt.megaton_assert(4 - 2, 2);
    mtt.megaton_assert(4 * 2, 8);
    mtt.megaton_assert(4 / 2, 2);

    // more in depth
    for i in 0..25 {
        for j in 0..25 {
            mtt.megaton_assert(i + j, j + i);
            mtt.megaton_assert(i * j, j * i);
            mtt.megaton_assert(((i - j) as i32).abs(), ((j - i) as i32).abs());
        }
        mtt.megaton_assert(i * 2, i + i);
        mtt.megaton_assert(i - i, 0)
    }
    mtt.end_category();
}

fn megaton_string_tests(mtt: &mut MegatonTests) {
    mtt.start_category("Strings");
    mtt.megaton_assert("", "");
    mtt.end_category();
}

fn run_megaton_tests() {
    let mut mtt = MegatonTests::new();
    megaton_num_tests(&mut mtt);
    megaton_string_tests(&mut mtt);
    fs_tests::megaton_file_tests(&mut mtt);
    mtt.megaton_log(format!("Tests finished, {:#?}/{:#?} Passed\n", mtt.passed_tests, mtt.total_tests).as_str());
}

#[unsafe(no_mangle)]
extern "C" fn __megaton_rs_main() {
    ffi::init_function_in_c();
}
