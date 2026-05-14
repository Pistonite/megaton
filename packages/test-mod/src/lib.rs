use std::{fs::File, io::Write, path::PathBuf};

pub use megaton::*;

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

struct MegatonTests<'a> {
    total_tests: usize,
    passed_tests: usize,
    category_tests: usize,
    category_passed_tests: usize,
    category: &'a str
}

impl<'a> MegatonTests<'a> {

    fn new() -> MegatonTests<'a> {
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

    fn start_category(&mut self, category: &'a str) {
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

fn megaton_file_tests(mtt: &mut MegatonTests) {
    mtt.start_category("Files");

    basic_tests(mtt);
    test_consecutive_writes(mtt);
    test_write_seek_offset(mtt);
    test_close_frees_fd(mtt);
    test_multiple_files(mtt);
    
    

    mtt.end_category();
}

fn basic_tests(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");
    const total_content: &[u8] = "Hello world!\nA".as_bytes();
    const lines: [&[u8]; 2] = ["Hello world!\n".as_bytes(), "A".as_bytes()];
    const total_len: usize = total_content.len();
    
    mtt.megaton_log("TEST: Testing exists!\n");
    if path.exists() {
        mtt.megaton_log("TEST: File exists, removing!\n");
        let result = std::fs::remove_file(&path);
        if mtt.megaton_assert_ok(result, "Failed to remove file!\n").is_none() {
            return;
        }
    }

    mtt.megaton_log("TEST: Creating test file\n");
    let result = File::create(&path);
    let result = mtt.megaton_assert_ok(result, "Failed to create file!\n");
    if result.is_none() {
        return;
    }

    let mut test_file = result.unwrap();
    let result = test_file.write(lines[0]);
    mtt.megaton_assert_ok(result, "Failed to write to file\n");
}

fn test_consecutive_writes(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");
    const total_content: &[u8] = "Hello world!\nA".as_bytes();
    const lines: [&[u8]; 2] = ["Hello world!\n".as_bytes(), "A".as_bytes()];

    let result = File::create(&path);
    let result = mtt.megaton_assert_ok(result, "Failed to create file!\n");

    if result.is_none() {
        return;
    }

    let mut test_file = result.unwrap();
    let result = test_file.write(lines[0]);
    mtt.megaton_assert_ok(result, "Failed to write to file\n");

    mtt.megaton_log("TEST: Testing consecutive writes append\n");
    let result = test_file.write(lines[1]);
    mtt.megaton_assert_ok(result, "Failed to write second chunk to file\n");

    let read_back = std::fs::read(&path);
    if let Some(content) = mtt.megaton_assert_ok(read_back, "Failed to read back file after consecutive writes\n") {
        mtt.megaton_assert_msg(content.as_slice(), total_content, "Consecutive writes did not append correctly");
    }
}

fn test_write_seek_offset(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");
    const lines: [&[u8]; 2] = ["Hello world!\n".as_bytes(), "A".as_bytes()];

    mtt.megaton_log("TEST: Testing write modifies seek offset\n");
    let result = File::create(&path);
    if let Some(mut file) = mtt.megaton_assert_ok(result, "Failed to recreate file for seek offset test\n") {
        let result = file.write(lines[0]);
        if mtt.megaton_assert_ok(result, "Failed to write for seek offset test\n").is_some() {
            let content = std::fs::read(&path);
            if let Some(bytes) = mtt.megaton_assert_ok(content, "Failed to read back file\n") {
                mtt.megaton_assert_msg(bytes.len(), lines[0].len(), "File length wrong after write");
            }
        }
    }
}

fn test_close_frees_fd(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");

    mtt.megaton_log("TEST: Testing close frees from fd list\n");
    let result = File::create(&path);
    if let Some(file) = mtt.megaton_assert_ok(result, "Failed to create file for close test\n") {
        drop(file);
        let result = File::open(&path);
        mtt.megaton_assert_ok(result, "Failed to reopen file after close - fd was not freed\n");
    }
}

fn test_multiple_files(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");
    let path2: PathBuf = PathBuf::from("sd:/testfile2.txt");
    let path3: PathBuf = PathBuf::from("sd:/testfile3.txt");

    mtt.megaton_log("TEST: Testing opening multiple files\n");
    let file1 = mtt.megaton_assert_ok(File::create(&path), "Failed to open file 1\n");
    let file2 = mtt.megaton_assert_ok(File::create(&path2), "Failed to open file 2\n");
    let file3 = mtt.megaton_assert_ok(File::create(&path3), "Failed to open file 3\n");
    mtt.megaton_assert(file1.is_some(), true);
    mtt.megaton_assert(file2.is_some(), true);
    mtt.megaton_assert(file3.is_some(), true);
}

fn run_megaton_tests() {
    let mut mtt = MegatonTests::new();
    megaton_num_tests(&mut mtt);
    megaton_string_tests(&mut mtt);
    megaton_file_tests(&mut mtt);
    mtt.megaton_log(format!("Tests finished, {:#?}/{:#?} Passed\n", mtt.passed_tests, mtt.total_tests).as_str());
}

#[unsafe(no_mangle)]
extern "C" fn __megaton_rs_main() {
    ffi::init_function_in_c();
}
