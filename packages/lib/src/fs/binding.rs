// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#[derive(Debug)]
#[repr(C)]
pub struct NNResult {
    pub success: bool,
    pub module: i32,
    pub description: i32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum FileDescriptorType {
    #[allow(dead_code)]
    File,
    #[allow(dead_code)]
    Directory,
    #[allow(dead_code)]
    Tcp,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct FileDescriptor {
    pub inner: u64,
    pub kind: FileDescriptorType,
    pub seek_offset: u64,
}

#[repr(C)]
pub struct OpenResult {
    pub result: NNResult,
    pub fd: FileDescriptor,
}

#[repr(C)]
pub struct ReadResult {
    pub result: NNResult,
    pub bytes_read: usize,
}

#[repr(C)]
pub struct GetEntryTypeResult {
    pub result: NNResult,
    pub entry_type: DirectoryEntryType,
}

#[repr(C)]
pub struct GetSizeResult {
    pub result: NNResult,
    pub size: i64,
}

unsafe extern "C" {
    #[link_name = "__megaton_lib_fs_write_file"]
    pub unsafe fn write_file(nn_fd: u64, buf: *const u8, size: usize, position: u64) -> NNResult;

    #[link_name = "__megaton_lib_fs_open"]
    pub unsafe fn open(name: *const i8, flags: i32, mode: i32) -> OpenResult;

    #[link_name = "__megaton_lib_fs_get_entry_type"]
    pub unsafe fn get_entry_type(name: *const i8) -> GetEntryTypeResult;

    #[link_name = "__megaton_lib_fs_get_file_size"]
    pub unsafe fn get_file_size(nn_fd: u64) -> GetSizeResult;

    #[link_name = "__megaton_lib_fs_read_file"]
    pub unsafe fn read_file(nn_fd: u64, seek_pos: u64, buf: *mut u8, len: u64) -> ReadResult;

    #[link_name = "__megaton_lib_fs_close_file"]
    pub unsafe fn close_file(nn_fd: u64);

    #[link_name = "__megaton_lib_fs_close_dir"]
    pub unsafe fn close_directory(nn_fd: u64);

    #[link_name = "__megaton_lib_fs_unlink"]
    pub unsafe fn unlink(name: *const i8) -> NNResult;

    #[link_name = "__megaton_lib_log"]
    pub unsafe fn megaton_log(buf: *const u8, len: u64);

    #[link_name = "__megaton_lib_fs_write_stdout"]
    pub unsafe fn write_stdout(buf: *const u8, len: u64);

    #[link_name = "__megaton_lib_fs_write_stderr"]
    pub unsafe fn write_stderr(buf: *const u8, len: u64);
}

#[allow(dead_code)] // FIXME
// https://github.com/hermit-os/hermit-rs/blob/82146cf059bf3894eea1e96beed9da72b99b9d5a/hermit-abi/src/lib.rs#L44
pub const STDIN_FILENO: usize = 0;
#[allow(dead_code)] // FIXME
pub const STDOUT_FILENO: usize = 1;
pub const STDERR_FILENO: usize = 2;
#[allow(dead_code)] // FIXME
pub const LOG_FILENO: usize = 3;

pub const O_RDONLY: i32 = 0o0;
#[allow(dead_code)] // FIXME
pub const O_WRONLY: i32 = 0o1;
#[allow(dead_code)] // FIXME
pub const O_RDWR: i32 = 0o2;
#[allow(dead_code)] // FIXME
pub const O_CREAT: i32 = 0o100;
#[allow(dead_code)] // FIXME
pub const O_EXCL: i32 = 0o200;
#[allow(dead_code)] // FIXME
pub const O_TRUNC: i32 = 0o1000;
#[allow(dead_code)] // FIXME
pub const O_APPEND: i32 = 0o2000;
#[allow(dead_code)] // FIXME
pub const O_NONBLOCK: i32 = 0o4000;
#[allow(dead_code)] // FIXME
pub const O_DIRECTORY: i32 = 0o200000;

#[allow(dead_code)] // FIXME
pub const FS_ERR_MODULE: i32 = 2; // all fs errors will have module = 2
#[allow(dead_code)] // FIXME
pub const PATH_NOT_FOUND: i32 = 1;
#[allow(dead_code)] // FIXME
pub const PATH_ALREADY_EXISTS: i32 = 2;
#[allow(dead_code)] // FIXME
pub const TARGET_LOCKED: i32 = 7;
#[allow(dead_code)] // FIXME
pub const DIRECTORY_NOT_EMPTY: i32 = 8;

/* Kinds of entries within a directory. */
#[repr(C)]
pub enum DirectoryEntryType {
    #[allow(dead_code)] // FIXME
    Directory,
    #[allow(dead_code)] // FIXME
    File,
}

/* Bitfield to define the kinds of entries to open from a directory. */
#[repr(C)]
#[allow(dead_code)] // FIXME
enum OpenDirectoryMode {
    #[allow(dead_code)] // FIXME
    Directory = 1 << 0,
    #[allow(dead_code)] // FIXME
    File = 1 << 1,
    #[allow(dead_code)] // FIXME
    All = (1 << 0) | (1 << 1),
}
