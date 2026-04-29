use std::{array, sync::{Arc, LazyLock, Mutex}};

const NUM_FDS: usize = 1000;
const GENERIC_ERRNO: i32 = -1;
// TODO: Allow user to configure stdio paths
const STDIN_PATH: &str = "sd:/megaton_stdin.txt";
const STDOUT_PATH: &str = "sd:/megaton_stdout.txt";
const STDERR_PATH: &str = "sd:/megaton_stderr.txt";

use crate::fs::fs_helpers::{self, FileDescriptor, FileDescriptorType, GetEntryTypeResult, O_CREAT, O_RDONLY, O_RDWR, O_TRUNC, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};

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

fn get_fd_entry(fd: usize) -> Option<FileDescriptor> {
    if fd < 0 || fd >= NUM_FDS {
        return None;
    }

    let fd = fd as usize;
    LIST.try_lock().unwrap()[fd]
}

pub fn init_stdio(){
    // stdio takes up 3 entries in the fd list, but makes the indexing logic simpler.
    // Otherwise, we need to offset every fd we give to the user, or given to us by the user, by 3. 
    // If we need more fds later, we can just make the list bigger.
    let list = &mut LIST.try_lock().unwrap();
    let stdin_path: *const i8 = STDIN_PATH.as_ptr() as *const i8;
    let stdout_path: *const i8 = STDOUT_PATH.as_ptr() as *const i8;
    let stderr_path: *const i8 = STDERR_PATH.as_ptr() as *const i8;
    
    // open for read/write, create it if the file doesn't exist, and delete all existing content if it does
    let stdin_fd = unsafe { fs_helpers::open(stdin_path, O_RDWR | O_CREAT | O_TRUNC, 0) };
    let stdout_fd = unsafe { fs_helpers::open(stdout_path, O_RDWR | O_CREAT | O_TRUNC, 0) };
    let stderr_fd = unsafe { fs_helpers::open(stderr_path, O_RDWR | O_CREAT | O_TRUNC, 0) };

    assert!(stdin_fd.result.success && stdout_fd.result.success && stderr_fd.result.success);
    list[STDIN_FILENO] = Some(stdin_fd.fd);
    list[STDOUT_FILENO] = Some(stdout_fd.fd);
    list[STDERR_FILENO] = Some(stderr_fd.fd);
}


#[unsafe(no_mangle)]
pub extern "C" fn sys_open(name: *const i8, flags: i32, mode: i32) -> i32 {
    // TODO: map flags and mode to nnheaders flags and mode
    let open_result = unsafe { fs_helpers::open(name, flags, mode) };
    if open_result.result.success  {
        match insert_into_fd_list(open_result.fd) {
            Some(fd_index) => fd_index as i32,
            None => GENERIC_ERRNO,
        }
    } else {
        assert!(open_result.result.module == fs_helpers::FS_ERR_MODULE);
        GENERIC_ERRNO
    }
}

#[unsafe(no_mangle)]
pub fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize {
    let fd_entry: &mut Option<FileDescriptor> = &mut get_fd_entry(fd as usize);
    match fd_entry {
        None => -1,
        Some(fd) => {
            match fd.kind {
                FileDescriptorType::FILE => try_write_file(fd, buf, len),
                FileDescriptorType::DIR => GENERIC_ERRNO as isize,
                FileDescriptorType::TCP => todo!(),
                FileDescriptorType::STDIN => try_write_file(fd, buf, len),
                FileDescriptorType::STDOUT => try_write_file(fd, buf, len),
                FileDescriptorType::STDERR => try_write_file(fd, buf, len),
            }
        }
    }
}

fn try_write_file(fd: &mut FileDescriptor, buf: *const u8, len: usize) -> isize {
    let write_result = unsafe { fs_helpers::write_file(fd.inner, fd.seek_offset, buf, len) };
    
    if write_result.success {
        fd.seek_offset += len as u64;
        len as isize
    } else {
        assert!(write_result.module == fs_helpers::FS_ERR_MODULE);
        match write_result.description {
            _ => GENERIC_ERRNO as isize // TODO: Map nn error to errno
        }
    }    
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_read(fd: i32, buf: *mut u8, len: usize) -> isize {
    let fd_entry: &mut Option<FileDescriptor> = &mut get_fd_entry(fd as usize);
    match fd_entry {
        None => -1,
        Some(fd) => {
            match fd.kind {
                FileDescriptorType::FILE => try_read_file(fd, buf, len),
                FileDescriptorType::DIR => todo!(),
                FileDescriptorType::TCP => todo!(),
                FileDescriptorType::STDIN => todo!(),
                FileDescriptorType::STDOUT => todo!(),
                FileDescriptorType::STDERR => todo!(),
            }
        }
    }
}

fn try_read_file(fd_entry: &mut FileDescriptor, buf: *mut u8, len: usize) -> isize {
    let read_result = unsafe { fs_helpers::read_file(fd_entry.inner, fd_entry.seek_offset, buf, len as u64) };
    if !read_result.result.success {
        assert!(read_result.result.module == fs_helpers::FS_ERR_MODULE);
        return GENERIC_ERRNO as isize
    } 
    read_result.bytes_read as isize
}

// https://github.com/hermit-os/hermit-rs/blob/111a7b480a18ce1b6c576d9dac02a688203432ee/hermit/src/syscall/mod.rs#L187
#[unsafe(no_mangle)]
pub extern "C" fn sys_stat(name: *const i8, stat: *mut fs_helpers::stat) -> i32 {
    let entry_type: GetEntryTypeResult = unsafe { fs_helpers::get_entry_type(name) };
    if !entry_type.result.success {
        assert!(entry_type.result.module == fs_helpers::FS_ERR_MODULE);
        match entry_type.result.description {
            fs_helpers::PATH_NOT_FOUND => {
                return GENERIC_ERRNO; // TODO: get err code for this
            },
            
            _ => {
                return GENERIC_ERRNO;  // TODO: get err code for this
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
                return GENERIC_ERRNO;  // TODO: get err code for this
            }
        }
    }

    let handle: FileDescriptor = if already_open {
        // TODO: get fd entry with matching name
        todo!()
    } else {
        open_result.fd
    };

    


    // TODO: check validity of stat?
    {
        let mut stat = unsafe { *stat };
        // TODO: implement
        stat.st_gid = 0;
        stat.st_dev = 0;
        stat.st_ino = 0;
        stat.st_nlink = 0;

        const S_IRUSR: u32 = 0o400;
        const S_IWUSR: u32 = 0o200;
        stat.st_mode = S_IRUSR | S_IWUSR;

        let size_result = unsafe { fs_helpers::get_file_size(handle.inner) };
        if !size_result.result.success {
            return GENERIC_ERRNO;
        } else {
            stat.st_size = size_result.size;
            stat.st_blksize = 1000;
            stat.st_blocks = ((size_result.size as f64) / 1000.0f64).ceil() as i64;
        }
        

    };

    GENERIC_ERRNO
}