use std::{array, sync::{Arc, LazyLock, Mutex}};

const NUM_FDS: usize = 1000;

use crate::fs_helpers::{self, FileDescriptor, FileDescriptorType};

static LIST: Mutex<[Option<FileDescriptor>; NUM_FDS]> = Mutex::new([None; NUM_FDS]);


fn insert_into_fd_list(fd: FileDescriptor) -> Option<usize> {
    let list = &mut LIST.try_lock().unwrap();
    for i in 0..NUM_FDS {
        if list[i].is_none() {
            list[i] = Some(fd);
            return Some(i);
        }
    }
    None
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_open(name: *const i8, flags: i32, mode: i32) -> i32 {
    // TODO: map flags and mode to nnheaders flags and mode
    let result = unsafe { fs_helpers::open(name, flags, mode) };
    if result.error_code != 0 {
        result.error_code
    } else {
        match insert_into_fd_list(result.fd) {
            Some(fd_index) => fd_index as i32,
            None => -1,
        }
    }
}


#[unsafe(no_mangle)]
pub fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize {
    if fd < 0 || fd >= (NUM_FDS as i32) {
        return -1;
    }

    let fd = fd as usize;
    let fd_entry: &mut Option<FileDescriptor> = &mut LIST.try_lock().unwrap()[fd];
    match fd_entry {
        None => -1,
        Some(fd) => {
            match fd.kind {
                FileDescriptorType::FILE => try_write_file(fd, buf, len),
                FileDescriptorType::DIR => todo!(),
                FileDescriptorType::TCP => -1,
                FileDescriptorType::STDIN => todo!(),
                FileDescriptorType::STDOUT => todo!(),
                FileDescriptorType::STDERR => todo!(),
            }
        }
    }
}


fn try_write_file(fd: &mut FileDescriptor,   buf: *const u8, len: usize) -> isize {
    let result = unsafe { fs_helpers::write_file(fd.inner, fd.seek_offset, buf, len) };
    if result > 0 {
        fd.seek_offset += result as u64;
    }
    result
}