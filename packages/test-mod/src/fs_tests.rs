use std::{fs::{File, OpenOptions}, io::{Read, Write}, path::PathBuf};

use crate::MegatonTests;

const LINES: [&[u8]; 2] = ["Hello world!\n".as_bytes(), "A".as_bytes()];
const TOTAL_CONTENT: &[u8] = "Hello world!\nA".as_bytes();
const STDOUT_PATH: &str = "sd:/megaton_stdout.txt";
const STDERR_PATH: &str = "sd:/megaton_stderr.txt";


pub fn megaton_file_tests(mtt: &mut MegatonTests) {
    mtt.start_category("Files");

    basic_tests(mtt);
    test_exists(mtt);
    test_consecutive_writes(mtt);
    test_write_seek_offset(mtt);
    test_close_frees_fd(mtt);
    test_multiple_files(mtt);
    test_read_seek_offset(mtt);
    test_open_flags(mtt);
    test_print(mtt);
    test_stderr(mtt);
    
    mtt.end_category();
}

fn basic_tests(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");
    const total_len: usize = TOTAL_CONTENT.len();
    
    if path.exists() {
        let result = std::fs::remove_file(&path);
        if mtt.megaton_assert_ok(result, "Test \"basic_tests\": Failed to remove file!\n").is_none() {
            return;
        }
    }

    let result = File::create(&path);
    let result = mtt.megaton_assert_ok(result, "Test \"basic_tests\": Failed to create file!\n");
    mtt.megaton_assert_msg(result.is_some(), true, "Test \"basic_tests\": File::create returned None, expected Some");
    if result.is_none() {
        return;
    }

    let mut test_file = result.unwrap();
    let result = test_file.write(LINES[0]);
    mtt.megaton_assert_ok(result, "Test \"basic_tests\": Failed to write to file\n");
}

fn test_print(mtt: &mut MegatonTests) {
    let content = "Hello from test_print!";
    println!("Hello from test_print!");
    let stdout_path = PathBuf::from(STDOUT_PATH);
    mtt.megaton_assert_msg(stdout_path.exists(), true, "Test \"test_print\": Failed to write to stdout");
    if !stdout_path.exists() {
        return;
    }

    let read_result = std::fs::read_to_string(stdout_path);
    if let Some(stdout_content) = mtt.megaton_assert_ok(read_result, "Test \"test_print\": Failed to read stdout") {
        mtt.megaton_assert_msg(stdout_content.contains(content), true, "Test \"test_print\": Content not written to stdout!");
    }
}

fn test_stderr(mtt: &mut MegatonTests) {
    let content = "This should go to stderr";
    dbg!(content);
    let stderr_path = PathBuf::from(STDERR_PATH);
    mtt.megaton_assert_msg(stderr_path.exists(), true, "Test \"test_stderr\": Failed to write to stderr");
    if !stderr_path.exists() {
        return;
    }

    let read_result = std::fs::read_to_string(stderr_path);
    if let Some(stdout_content) = mtt.megaton_assert_ok(read_result, "Test \"test_stderr\": Failed to read stderr") {
        mtt.megaton_assert_msg(stdout_content.contains(content), true, "Test \"test_stderr\": Failed to find expected error in stderr");
    }
}

fn test_consecutive_writes(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/test_consecutive_writes.txt");

    let open_result = 
        OpenOptions::new()
            .create(true).read(true).write(true)
            .open(&path);
    let result = mtt.megaton_assert_ok(open_result, "Test \"test_consecutive_writes\": Failed to create file!\n");
    mtt.megaton_assert(result.is_some(), true);
    if result.is_none() {
        return;
    }

    let mut test_file = result.unwrap();
    let result = test_file.write(LINES[0]);
    mtt.megaton_assert_ok(result, "Test \"test_consecutive_writes\": Failed to write to file\n");

    let result = test_file.write(LINES[1]);
    mtt.megaton_assert_ok(result, "Test \"test_consecutive_writes\": Failed to write second chunk to file\n");

    let read_back = std::fs::read(&path);
    if let Some(content) = mtt.megaton_assert_ok(read_back, "Test \"test_consecutive_writes\": Failed to read back file after consecutive writes\n") {
        mtt.megaton_assert_msg(content.as_slice(), TOTAL_CONTENT, "Test \"test_consecutive_writes\": Consecutive writes did not append correctly");
    }
}

fn test_write_seek_offset(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");

    let result = File::create(&path);
    if let Some(mut file) = mtt.megaton_assert_ok(result, "Test \"test_write_seek_offset\": Failed to recreate file for seek offset test\n") {
        let result = file.write(LINES[0]);
        if mtt.megaton_assert_ok(result, "Test \"test_write_seek_offset\": Failed to write for seek offset test\n").is_some() {
            let content = std::fs::read(&path);
            if let Some(bytes) = mtt.megaton_assert_ok(content, "Test \"test_write_seek_offset\": Failed to read back file\n") {
                mtt.megaton_assert_msg(bytes.len(), LINES[0].len(), "Test \"test_write_seek_offset\": File length wrong after write");
            }
        }
    }
}

fn test_exists(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/shouldnt_exist.txt");
    let _result = std::fs::remove_file(&path);
    mtt.megaton_assert_msg(path.exists(), false, "Test \"test_exists\": Expected file not to exist");

    let path2 = PathBuf::from("sd:/should_exist.txt");
    let create_result = File::create(&path2);
    mtt.megaton_assert_msg(path2.exists(), true, "Test \"test_exists\": Expected file to exist");
}

fn test_close_frees_fd(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");
    let result = File::create(&path);
    if let Some(file) = mtt.megaton_assert_ok(result, "Test \"test_close_frees_fd\": Failed to create file for close test\n") {
        drop(file);
        let result = File::open(&path);
        mtt.megaton_assert_ok(result, "Test \"test_close_frees_fd\":  Failed to reopen file after close - fd was not freed\n");
    }

    std::fs::remove_file(path);
}

fn test_multiple_files(mtt: &mut MegatonTests) {
    let path: PathBuf = PathBuf::from("sd:/testfile.txt");
    let path2: PathBuf = PathBuf::from("sd:/testfile2.txt");
    let path3: PathBuf = PathBuf::from("sd:/testfile3.txt");

    let file1 = mtt.megaton_assert_ok(File::create(&path), "Test \"test_multiple_files\": Failed to open file 1\n");
    let file2 = mtt.megaton_assert_ok(File::create(&path2), "Test \"test_multiple_files\": Failed to open file 2\n");
    let file3 = mtt.megaton_assert_ok(File::create(&path3), "Test \"test_multiple_files\": Failed to open file 3\n");
    mtt.megaton_assert(file1.is_some(), true);
    mtt.megaton_assert(file2.is_some(), true);
    mtt.megaton_assert(file3.is_some(), true);

    std::fs::remove_file(path);
    std::fs::remove_file(path2);
    std::fs::remove_file(path3);
}

fn test_read_seek_offset(mtt: &mut MegatonTests) {

    let path: PathBuf = PathBuf::from("sd:/test_read_seek.txt");

    let result = File::create(&path);
    if result.is_err() {
        return;
    }
    let mut file = result.unwrap();
    if mtt.megaton_assert_ok(file.write(TOTAL_CONTENT), "Failed to write to file\n").is_none() {
        return;
    }
    drop(file);

    let result = File::open(&path);
    if let Some(mut file) = mtt.megaton_assert_ok(result, "Failed to open file for read seek test\n") {
        let mut buf1 = vec![0u8; LINES[0].len()];
        let result = file.read(&mut buf1);
        if mtt.megaton_assert_ok(result, "Failed to read first chunk\n").is_some() {
            mtt.megaton_assert_msg(buf1.as_slice(), LINES[0], "First read got wrong content");
            let mut buf2 = vec![0u8; LINES[1].len()];
            let result = file.read(&mut buf2);
            if mtt.megaton_assert_ok(result, "Failed to read second chunk\n").is_some() {
                mtt.megaton_assert_msg(buf2.as_slice(), LINES[1], "Second read got wrong content");
            }
        }
    }

    std::fs::remove_file(path);
}

fn test_open_flags(mtt: &mut MegatonTests) {

    let path: PathBuf = PathBuf::from("sd:/test_open_flags.txt");

    // O_CREAT should create a new file
    let result = File::create(&path);
    mtt.megaton_assert_ok(result, "Test \"test_open_flags\": O_CREAT failed: could not create file\n");

    // O_TRUNC should truncate on existing file
    let result = File::create(&path);
    if let Some(mut file) = mtt.megaton_assert_ok(result, "Test \"test_open_flags\": Failed to open file for truncate test\n") {
        file.write(TOTAL_CONTENT);
        drop(file);
        let result = File::create(&path); // truncates
        if let Some(_) = mtt.megaton_assert_ok(result, "Test \"test_open_flags\": O_TRUNC failed: could not truncate file\n") {
            let bytes = std::fs::read(&path);
            if let Some(bytes) = mtt.megaton_assert_ok(bytes, "Test \"test_open_flags\": Failed to read after truncate\n") {
                mtt.megaton_assert_msg(bytes.len(), 0, "Test \"test_open_flags\": O_TRUNC did not truncate file to zero");
            }
        }
    }

    // O_RDONLY should not allow writing on file
    let result = File::open(&path);
    if let Some(mut file) = mtt.megaton_assert_ok(result, "Test \"test_open_flags\": O_RDONLY failed: could not open file for reading\n") {
        let write_result = file.write(TOTAL_CONTENT);
        mtt.megaton_assert_msg(write_result.is_err(), true, "Test \"test_open_flags\": Writing to file opened with O_RDONLY should fail!");
    }
}