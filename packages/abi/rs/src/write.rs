// // SPDX-License-Identifier: GPL-3.0-or-later
// // Copyright (c) 2025 Megaton contributors


// use std::{borrow::BorrowMut, collections::HashMap, cell::LazyCell, fs::File, path::Path, sync::{Mutex, Once}};
// use std::sync::LazyLock;
// use nn::fs::

use std::{ffi::{CStr, c_char}, sync::{LazyLock, Mutex}};

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

    // type Result = u64;

    #[namespace = "fs"] 
    unsafe extern "C++" {
        include!("write.h");
        unsafe fn open_file(name: &str, flags: i32, mode: i32) -> u64;
        // unsafe fn close_directory(handle: u64);
        fn close_file(handle: u64);
        fn open_dir(name: &str) -> u64;
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
    FILE(u64),
    TCP(u64),
    STDIN,
    STDOUT,
    STDERR,
    UNUSED,
}
// TODO: Separate into multiple lists
// Uhyve, Fuse: probably don't need to implement these
// eventfd: Not used in rust stdlib
// 

struct FDList([FD; NUM_FDS]);
pub const NUM_FDS: usize = 1024;
type FDLock = LazyLock<Mutex<FDList>>; // TODO: init at initialization time, dont use lazylock
static FDS: FDLock = LazyLock::new(|| {
    let mut fdlist = [FD::UNUSED; NUM_FDS];
    fdlist[0] = FD::STDIN;
    fdlist[1] = FD::STDOUT;
    fdlist[2] = FD::STDERR;
    Mutex::new(FDList(fdlist))
});

fn add_fd(fd: FD) -> usize {
    let mut fds = FDS.lock().expect("Unable to acquire FDList mutex lock!");
    let new_fd_index = fds.0.iter().position(|fd| *fd == FD::UNUSED ).expect("FDList is full!");
    fds.0[new_fd_index] = fd;

    new_fd_index
}

fn remove_fd(fd: usize) {
    let mut fds = FDS.lock().expect("Unable to acquire FDList mutex lock!");
    let new_fd_index = fds.0.iter().position(|fd| *fd == FD::UNUSED ).expect("FDList is full!");
    fds.0[new_fd_index] = FD::UNUSED;
}

fn get_fd(user_fd: usize) -> FD {
    let fds = FDS.lock().expect("Unable to acquire FDList mutex lock!");
    fds.0[user_fd].clone() // todo: replace with reference?
}

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
pub extern "C" fn sys_open(name: *const c_char, flags: i32, mode: i32) -> i32 {
    let new_filefd: FD;
    let flags = get_flags(flags);
    let mode = get_mode(mode);
    if let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str() {
		let res = unsafe { ffi::open_file(name, flags, mode) };
        new_filefd = FD::FILE(res);
        add_fd(new_filefd) as i32
	} else {
        // TODO: Import Errno from hermit kernel https://github.com/hermit-os/kernel/blob/main/src/errno.rs#L126
		-22 // Errno::INVAL 
	}
    
}

// #[unsafe(no_mangle)]
// pub extern "C" fn sys_open(name: *const i8, flags: i32, mode: i32) -> i32 {
//     let new_filefd: FD;
//     let flags = get_flags(flags);
//     let mode = get_mode(mode);
//     unsafe {
//         let res = ffi::open_file(name, flags, mode);
//         new_filefd = FD::FILE(res);
//     }
//     add_fd(new_filefd) as i32
// }


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
    let elem = get_fd(fd as usize);
    match elem {
        FD::FILE(i) => {
            ffi::close_file(i as u64);
        }
        FD::TCP(i) => todo!(),
        FD::STDIN => todo!(),
        FD::STDOUT => todo!(),
        FD::STDERR => todo!(),
        FD::UNUSED => panic!("Tried to close unallocated FD #{:?} : {:?}", fd, elem),
    }

    return 0;
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_rmdir(_name: *const i8) -> i32 {
	todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_fd() {
        let fd = FD::FILE(92);

        let res = add_fd(fd);
        assert!(res == 3);

        let f = &FDS;
        let f = f.lock().unwrap();
        let inserted = get_fd(res);
        assert_eq!(inserted, fd, "{:#<1?}\n Contents of FDList are above.", f.0);
    }

}