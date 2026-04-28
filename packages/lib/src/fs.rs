use std::{array, sync::{Arc, LazyLock, Mutex}};

const NUM_FDS: usize = 1000;

use crate::fs_helpers::{self, FileDescriptor, FileDescriptorType, GetEntryTypeResult, O_RDONLY};

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
    let open_result = unsafe { fs_helpers::open(name, flags, mode) };
    if open_result.result.description != 0 {
        open_result.result.description
    } else {
        match insert_into_fd_list(open_result.fd) {
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


fn try_write_file(fd: &mut FileDescriptor, buf: *const u8, len: usize) -> isize {
    let result = unsafe { fs_helpers::write_file(fd.inner, fd.seek_offset, buf, len) };
    if result > 0 {
        fd.seek_offset += result as u64;
    }
    result
}

// https://github.com/hermit-os/hermit-rs/blob/111a7b480a18ce1b6c576d9dac02a688203432ee/hermit/src/syscall/mod.rs#L187
#[unsafe(no_mangle)]
pub extern "C" fn sys_stat(name: *const i8, stat: *mut fs_helpers::stat) -> i32 {
    let entry_type: GetEntryTypeResult = unsafe { fs_helpers::get_entry_type(name) };
    if !entry_type.result.success {
        assert!(entry_type.result.module == fs_helpers::FS_ERR_MODULE);
        match entry_type.result.description {
            fs_helpers::PATH_NOT_FOUND => {
                return -1; // TODO: get err code for this
            },
            
            _ => {
                return -1;  // TODO: get err code for this
            }
        }
    }

    let mut already_open = false;
    let open_result = unsafe{ fs_helpers::open(name,0, O_RDONLY) };
    if !open_result.result.success {
        assert!(open_result.result.module == fs_helpers::FS_ERR_MODULE);
        match open_result.result.description {
            fs_helpers::TARGET_LOCKED => {
                already_open = true;
            },
            _ => {
                return -1;  // TODO: get err code for this
            }
        }
    }

    if already_open {
        // TODO: get fd entry with matching name
        
    }

    // TODO: check validity of stat?
    {
        let stat = unsafe { *stat };
        // TODO: implement

    };

    -1
}