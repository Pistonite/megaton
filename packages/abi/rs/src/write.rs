// // SPDX-License-Identifier: GPL-3.0-or-later
// // Copyright (c) 2025 Megaton contributors

use std::{ffi::{CStr, c_char}, sync::{LazyLock, Mutex}};

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

fn get_mode(unix_mode: u32) -> u32 {
    // todo: implement
    // map unix_mode => nn mode 
    // according to https://switchbrew.github.io/libnx/fs_8h.html#a0cbe318e03a1a66cd6e395254a05d60b
    unix_mode 
}

#[cxx::bridge]
mod ffi {
    #[namespace = "fs"] 
    unsafe extern "C++" {
        include!("write.h");
        unsafe fn open_file(name: &str, flags: i32, mode: u32) -> u64;
        // unsafe fn close_directory(handle: u64);
        fn close_file(handle: u64);
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn sys_open(name: *const c_char, flags: i32, mode: u32) -> FileDescriptor {
    let new_filefd: FD;
    let flags = get_flags(flags);
    let mode = get_mode(mode);
    if let Ok(name) = unsafe { CStr::from_ptr(name as *const c_char) }.to_str() {
		let res = unsafe { ffi::open_file(name, flags, mode) };
        new_filefd = FD::FILE(res);
        add_fd(new_filefd) as i32
	} else {
        // TODO: Import Errno from hermit kernel https://github.com/hermit-os/kernel/blob/main/src/errno.rs#L126
		-22 // Errno::INVAL 
	}
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_writev(fd: i32, iov: *const u8, iovcnt: usize) -> isize {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_close(fd: FileDescriptor) -> i32 {
    let elem = get_fd(fd as usize);
    match elem {
        FD::FILE(i) => {
            ffi::close_file(i as u64);
            remove_fd(fd as usize);
        }
        FD::TCP(i) => todo!(),
        FD::STDIN => todo!(),
        FD::STDOUT => todo!(),
        FD::STDERR => todo!(),
        FD::UNUSED => panic!("Tried to close unallocated FD #{:?} : {:?}", fd, elem),
    }

    return 0;
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