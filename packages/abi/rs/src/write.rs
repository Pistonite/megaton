// // SPDX-License-Identifier: GPL-3.0-or-later
// // Copyright (c) 2025 Megaton contributors

use std::{ffi::{CStr, c_char}, fs::File, sync::{LazyLock, Mutex}};

// TODO: type defined here https://github.com/hermit-os/kernel/blob/ec0fc3572c9d8deba725df8b7eb000980034a9f6/src/fd/mod.rs#L51
// but is this true? is it not system dependent
type FileDescriptor = i32; 

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FD {
    FILE(u64),
    TCP(u64),
    DIR(u64),
    STDIN,
    STDOUT,
    STDERR,
    UNUSED,
}

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

fn add_fd(fd: FD) -> FileDescriptor {
    let mut fds = FDS.lock().expect("Unable to acquire FDList mutex lock!");
    let new_fd_index = fds.0.iter().position(|fd| *fd == FD::UNUSED ).expect("FDList is full!");
    fds.0[new_fd_index] = fd;

    new_fd_index as FileDescriptor
}

fn remove_fd(fd: FileDescriptor) {
    let mut fds = FDS.lock().expect("Unable to acquire FDList mutex lock!");
    fds.0[fd as usize] = FD::UNUSED;
}

fn get_fd(user_fd: FileDescriptor) -> FD {
    let fds = FDS.lock().expect("Unable to acquire FDList mutex lock!");
    fds.0[user_fd as usize].clone()
}


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
        fn open_dir(name: &str) -> u64;
        fn close_dir(handle: u64) -> u64;
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn sys_open(name: *const c_char, flags: i32, mode: u32) -> FileDescriptor {
    let flags = get_flags(flags);
    let mode = get_mode(mode);
    if let Ok(name) = unsafe { CStr::from_ptr(name as *const c_char) }.to_str() {
		let res = unsafe { ffi::open_file(name, flags, mode) };
        let new_filefd = FD::FILE(res);
        add_fd(new_filefd) as FileDescriptor
	} else {
        // TODO: Import Errno from hermit kernel https://github.com/hermit-os/kernel/blob/main/src/errno.rs#L126
		-22 // Errno::INVAL 
	}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_opendir(name: *const c_char) -> FileDescriptor {
    if let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str() {
        let res = ffi::open_dir(name);
        let new_filefd = FD::DIR(res);
        add_fd(new_filefd) as FileDescriptor
    } else {
        -22
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_write(fd: FileDescriptor, buf: *const u8, len: usize) -> isize {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_writev(fd: FileDescriptor, iov: *const u8, iovcnt: usize) -> isize {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_close(fd: FileDescriptor) -> FileDescriptor {
    let elem = get_fd(fd);
    match elem {
        FD::FILE(i) => {
            ffi::close_file(i);
            remove_fd(fd);
        },
        FD::DIR(i) => {
            ffi::close_dir(i);
            remove_fd(fd);
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

        let inserted = get_fd(res);
        let f = &FDS;
        let f = f.lock().unwrap();
        assert_eq!(inserted, fd, "{:#<1?}\n Contents of FDList are above.", f.0);
    }

    #[test]
    fn test_remove_fd() {
        let fd = FD::FILE(92);

        let fd_index = add_fd(fd);
        remove_fd(fd_index);
        let result = get_fd(fd_index);
        let f = &FDS;
        let f = f.lock().unwrap();
        assert_eq!(result, FD::UNUSED, "{:#<1?}\n Contents of FDList are above.", f.0);
    }

}