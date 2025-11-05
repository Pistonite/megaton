// // SPDX-License-Identifier: GPL-3.0-or-later
// // Copyright (c) 2025 Megaton contributors


// use std::{borrow::BorrowMut, collections::HashMap, cell::LazyCell, fs::File, path::Path, sync::{Mutex, Once}};
// use std::sync::LazyLock;
// use nn::fs::

use std::sync::{LazyLock, Mutex};

mod abi {

    // taken from 
    #[allow(non_camel_case_types)]
    pub type time_t = i64;
    
    // https://github.com/hermit-os/hermit-rs/blob/82146cf059bf3894eea1e96beed9da72b99b9d5a/hermit-abi/src/lib.rs#L72
    #[derive(Default, Copy, Clone, Debug)]
    #[repr(C)]
    pub struct timespec {
        /// seconds
        pub tv_sec: time_t,
        /// nanoseconds
        pub tv_nsec: i32,
    }

    // https://github.com/hermit-os/hermit-rs/blob/82146cf059bf3894eea1e96beed9da72b99b9d5a/hermit-abi/src/lib.rs#L285
    #[repr(C)]
    #[derive(Debug, Default, Copy, Clone)]
    pub struct stat {
        pub st_dev: u64,
        pub st_ino: u64,
        pub st_nlink: u64,
        /// access permissions
        pub st_mode: u32,
        /// user id
        pub st_uid: u32,
        /// group id
        pub st_gid: u32,
        /// device id
        pub st_rdev: u64,
        /// size in bytes
        pub st_size: i64,
        /// block size
        pub st_blksize: i64,
        /// size in blocks
        pub st_blocks: i64,
        /// time of last access
        pub st_atim: timespec,
        /// time of last modification
        pub st_mtim: timespec,
        /// time of last status change
        pub st_ctim: timespec,
    }
}


#[cxx::bridge]
mod ffi {

    #[namespace = "fs"] 
    unsafe extern "C++" {
        include!("write.h");
        unsafe fn open_file(name: *const i8, flags: i32, mode: i32) -> u64;
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn sys_write(fd: u32, buf: &[u8], len: usize) -> isize {
    todo!()
    // write_file() from write.cpp
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_writev(fd: i32, iov: *const u8, iovcnt: usize) -> isize {
    todo!()
}

enum OpenMode {
    READ,
    WRITE,
    APPEND
}
impl Into<i32> for OpenMode {
    fn into(self) -> i32 {
        match self {
            OpenMode::READ => 1,
            OpenMode::WRITE => 2,
            OpenMode::APPEND => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FD {
    FILE(u32),
    TCP(u32),
    STDIO(u32),
    UNUSED,
}


struct FDList([FD; NUM_FDS]);
pub const NUM_FDS: usize = 1000;
type FDLock = LazyLock<Mutex<FDList>>;
static FDS: FDLock = LazyLock::new(|| {
    Mutex::new(FDList([FD::UNUSED; NUM_FDS]))
});


type FileDescriptor = i32;

fn get_flags(unix_flags: i32) -> i32 {
    // todo: implement
    // map unix_flags => nn flags
    unix_flags
}

fn get_mode(unix_mode: i32) -> i32 {
    // todo: implement
    // map unix_mode => nn mode 
    // according to https://switchbrew.github.io/libnx/fs_8h.html#a0cbe318e03a1a66cd6e395254a05d60b
    unix_mode 
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_open(name: *const i8, flags: i32, mode: i32) -> i32 {
    let mut new_filefd: Option<FD> = None;
    let flags = get_flags(flags);
    let mode = get_mode(mode);
    unsafe {
        // todo: instrument code to mock out open_file for testing purposes
        // let res = 64; // used in testing
        let res = ffi::open_file(name, flags, mode);
        new_filefd = Some(FD::FILE(res as u32));
    }
    if let Some(new_fd) = new_filefd {
        let mut fds = FDS.lock().expect("Unable to acquire mutex lock!");
        let new_fd_index = 0;
        fds.0[new_fd_index] = new_filefd.unwrap();

        new_fd_index as i32 // future calls to sys_write referencing new_fd will be handled by new_filefd
    } else {
        0
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn sys_fstat(_fd: i32, _stat: *mut abi::stat) -> i32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_lseek(fd: FileDescriptor, offset: isize, whence: i32) -> isize {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_close(fd: FileDescriptor) -> i32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_rmdir(_name: *const i8) -> i32 {
	todo!()
}

#[cfg(test)]
mod tests {
    // use std::fd::{OpenOption, AccessPermission};
    use std::{ffi::{CStr, CString}, path::PathBuf};
    // use std::fs::OpenOptions;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_init() {
        // TODO: How to stub out the call to ffi::open_file
        {
            let f = &FDS;
            let f = f.lock().unwrap();
            println!("{:#<1?}", f.0);
            let mut p = PathBuf::new();
            p.push("foobar.txt");

        }

        let mut fd: Option<i32> = None;
        unsafe {
            let f = CString::new("hello.txt").unwrap();
            let file_name: *const i8 = f.into_raw();
            // let flags = OpenOption::O_RDWR;
            // let perms = AccessPermission::default();
            let res = sys_open(file_name, 0o02, OpenMode::READ as i32 | OpenMode::WRITE as i32 );
            fd = Some(res);
        }

        assert!(fd.is_some());
        {
            let f = &FDS;
            let f = f.lock().unwrap();
            let inserted = f.0[fd.unwrap() as usize];
            println!("{:#<1?}", f.0);
            println!("{:?}", inserted);
            assert!(inserted == FD::FILE(64));
        }
    }
}