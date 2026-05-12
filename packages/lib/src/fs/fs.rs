use std::{ffi::CString, sync::{Mutex, MutexGuard}};

const NUM_FDS: usize = 1000;
const GENERIC_ERRNO: i32 = -1;
// TODO: Allow user to configure stdio paths
const STDIN_PATH: &[u8] = b"sd:/megaton_stdin.txt\0";
const STDOUT_PATH: &[u8] = b"sd:/megaton_stdout.txt\0";
const STDERR_PATH: &[u8] = b"sd:/megaton_stderr.txt\0";
const LOG_PATH: &[u8] = b"sd:/megaton_logs.txt\0";

use crate::fs::fs_helpers::{self, FileDescriptor, FileDescriptorType, GetEntryTypeResult, LOG_FILENO, NNResult, O_CREAT, O_RDONLY, O_RDWR, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, init_cpp_logging};

// Calling convention:
//  To avoid double-locking, only top-level exported functions (just syscalls for now) are allowed to lock the list.
//  Every other function should take a ListRef as needed.
static LIST: Mutex<[Option<FileDescriptor>; NUM_FDS]> = Mutex::new([None; NUM_FDS]);
type ListRef<'a> = MutexGuard<'a, [Option<FileDescriptor>; 1000]>;


fn insert_into_fd_list(list: &mut ListRef, fd: FileDescriptor) -> Option<usize> {
    for i in 3..NUM_FDS {
        if list[i].is_none() {
            list[i] = Some(fd);
            return Some(i);
        }
    }
    None
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_show_fd_list(){
    let list = LIST.try_lock().unwrap();
    for i in 0..NUM_FDS {
        if let Some(fd) = list[i] {
            // unsafe { sendf(b"%d=>{type=? inner=%u}\t\0".as_ptr() as *const std::ffi::c_char, fd.inner) };
        }
    }
}

fn get_fd_entry(list: &ListRef, fd: usize) -> Option<FileDescriptor> {
    if fd >= NUM_FDS {
        return None;
    }

    let fd = fd as usize;
    list[fd]
}

#[allow(dead_code)]
pub fn initialize_fs(){
    // stdio takes up 3 entries in the fd list, but makes the indexing logic simpler.
    // Otherwise, we need to offset every fd we give to the user, or given to us by the user, by 3. 
    // If we need more fds later, we can just make the list bigger.
    let list: &mut ListRef = &mut LIST.try_lock().unwrap();
    let stdin_path: *const i8 = STDIN_PATH.as_ptr() as *const i8; // TODO: Convert to C String.
    let stdout_path: *const i8 = STDOUT_PATH.as_ptr() as *const i8;
    let stderr_path: *const i8 = STDERR_PATH.as_ptr() as *const i8;
    let log_path: *const i8 = LOG_PATH.as_ptr() as *const i8;
    
    // open for read/write, create it if the file doesn't exist, and delete all existing content if it does
    let stdin_fd = unsafe { fs_helpers::open(stdin_path, O_RDWR | O_CREAT, 0) };
    let stdout_fd = unsafe { fs_helpers::open(stdout_path, O_RDWR | O_CREAT, 0) };
    let stderr_fd = unsafe { fs_helpers::open(stderr_path, O_RDWR | O_CREAT, 0) };
    let log_fd = unsafe { fs_helpers::open(log_path, O_RDWR | O_CREAT, 0) };

    if stdin_fd.result.success {
        list[STDIN_FILENO] = Some(stdin_fd.fd);
    } 
    if stdout_fd.result.success {
        list[STDOUT_FILENO] = Some(stdout_fd.fd);
    }
    if stderr_fd.result.success {
        list[STDERR_FILENO] = Some(stderr_fd.fd);
    }
    if log_fd.result.success {
        list[LOG_FILENO] = Some(log_fd.fd);
    }

    unsafe { init_cpp_logging(stderr_fd.fd.inner) };

}

// unsafe extern "C" {
//     // include!("toolkit/tcp.hpp");
//     // #[namespace = "botw::tcp"]
//     #[link_name = "_ZN4botw3tcp5sendfEPKcz"]
//     unsafe fn sendf(format: *const std::ffi::c_char, ...);
// }

fn megaton_log(list: &mut ListRef, msg: &str) {
    let len = (&msg).len();
    let msg = CString::new(msg).unwrap();
    
    let log_fd = get_fd_entry(list, LOG_FILENO);
    if let Some(mut log) = log_fd {
        try_write_file(&mut log, msg.as_c_str().as_ptr() as *const u8, len);
    }
}

fn write_stderr(list: &mut ListRef, msg: &str, result: &NNResult) {   
    // unsafe { sendf(b"Last result's description was %d\n\0".as_ptr() as *const std::ffi::c_char, result.description); }
    try_init_stderr(list);
    let mut stderr = get_fd_entry(list, STDERR_FILENO);
    try_write_file(stderr.as_mut().unwrap(), msg.as_ptr(), msg.len());
}

fn try_init_stderr(list: &mut ListRef) {
    let mut stderr = get_fd_entry(&list, STDERR_FILENO);

    if stderr.is_none() {
        // unsafe { sendf(b"opening stderr!\n\0".as_ptr() as *const std::ffi::c_char) };
        let stderr_fd = unsafe { fs_helpers::open(STDERR_PATH.as_ptr() as *const i8, O_RDWR | O_CREAT, 0) };
        if stderr_fd.result.success {
            stderr = Some(stderr_fd.fd);
            (*list)[STDERR_FILENO] = stderr;
            
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn sys_open(name: *const i8, flags: i32, mode: i32) -> i32 {
    // TODO: map flags and mode to nnheaders flags and mode
    // write_stderr(format!("Megaton: sys_open called! Args {:?} {} {}", name, flags, mode).as_str());
    
    let list: &mut ListRef = &mut LIST.try_lock().unwrap();
    megaton_log(list,format!("sys_open called with {:#?} {} {}\n", name, flags, mode).as_str());
    // megaton_log(list, "sys_open!! \0");

    let open_result = unsafe { fs_helpers::open(name, flags, mode) };
    if open_result.result.success  {
        match insert_into_fd_list(list, open_result.fd) {
            Some(fd_index) => fd_index as i32,
            None => GENERIC_ERRNO,
        }
    } else {
        // write_stderr(list, "Megaton: sys_open failed!\n", &open_result.result);
        // megaton_log(list, format!("Result was module={} description={}\n", open_result.result.module, open_result.result.description).as_str()) }// 
        GENERIC_ERRNO
    }
}

#[unsafe(no_mangle)]
pub fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize {
    // unsafe { sendf(b"sys_write called with %d %s %u\n\0".as_ptr() as *const std::ffi::c_char, fd, buf, len) }
    let list: &mut ListRef = &mut LIST.try_lock().unwrap();
    let fd_entry: &mut Option<FileDescriptor> = &mut get_fd_entry(list, fd as usize);
    match fd_entry {
        None => -1,
        Some(fd) => {
            match fd.kind {
                FileDescriptorType::FILE => try_write_file(fd, buf, len),
                FileDescriptorType::DIR => GENERIC_ERRNO as isize,
                FileDescriptorType::TCP => todo!(),
                FileDescriptorType::STDIN => try_write_file(fd, buf, len),
                FileDescriptorType::STDOUT => try_write_file(fd, buf, len),
                FileDescriptorType::STDERR => unsafe { 
                    // sendf("stderr: ".as_ptr() as *const std::ffi::c_char);
                    // sendf(buf as *const std::ffi::c_char);
                    try_write_file(fd, buf, len)
                },
            }
        }
    }
}


fn try_write_file(fd: &mut FileDescriptor, buf: *const u8, len: usize) -> isize {
    // unsafe { sendf(b"try_write_file %d %s\n\0".as_ptr() as *const std::ffi::c_char, fd.inner, buf) };
    let write_result = unsafe { fs_helpers::write_file(fd.inner, buf,  len, fd.seek_offset,) };
    
    if write_result.success {
        fd.seek_offset += len as u64;
        len as isize
    } else {
        // unsafe { sendf(b"Megaton: sys_write failed with %d\n\0".as_ptr() as *const std::ffi::c_char, write_result.description); }
        // assert!(write_result.module == fs_helpers::FS_ERR_MODULE);
        match write_result.description {
            _ => GENERIC_ERRNO as isize // TODO: Map nn error to errno
        }
    }    
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_read(fd: i32, buf: *mut u8, len: usize) -> isize {
    let list: &mut ListRef = &mut LIST.try_lock().unwrap();
    let fd_entry: &mut Option<FileDescriptor> = &mut get_fd_entry(list, fd as usize);
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
        // unsafe { sendf(b"Result was module=%d description=%d\n\0".as_ptr() as *const std::ffi::c_char, read_result.result.module, read_result.result.description) }
        // assert!(read_result.result.module == fs_helpers::FS_ERR_MODULE);
        return GENERIC_ERRNO as isize
    } 
    read_result.bytes_read as isize
}

// https://github.com/hermit-os/hermit-rs/blob/111a7b480a18ce1b6c576d9dac02a688203432ee/hermit/src/syscall/mod.rs#L187
#[unsafe(no_mangle)]
pub extern "C" fn sys_stat(name: *const i8, stat: *mut fs_helpers::stat) -> i32 {
    let entry_type: GetEntryTypeResult = unsafe { fs_helpers::get_entry_type(name) };
    if !entry_type.result.success {
        // unsafe { sendf(b"Result was module=%d description=%d\n\0".as_ptr() as *const std::ffi::c_char, entry_type.result.module, entry_type.result.description) }
        // assert!(entry_type.result.module == fs_helpers::FS_ERR_MODULE);
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
        // unsafe { sendf(b"sys_stat failed! Result was module=%d description=%d\n\0".as_ptr() as *const std::ffi::c_char, open_result.result.module, open_result.result.description) }
        // assert!(open_result.result.module == fs_helpers::FS_ERR_MODULE);
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
            try_close_file(handle.inner);
            return GENERIC_ERRNO;
        } else {
            stat.st_size = size_result.size;
            stat.st_blksize = 1000;
            stat.st_blocks = ((size_result.size as f64) / 1000.0f64).ceil() as i64;
            try_close_file(handle.inner);
            return 0;
        }
    };
}

#[unsafe(no_mangle)]
pub fn sys_close(fd: i32) -> i32 {
    // unsafe { sendf(b"Megaton: Closing file %d!\n\0".as_ptr() as *const std::ffi::c_char, fd) };
    let list: &mut ListRef = &mut LIST.try_lock().unwrap();
    let fd_entry: &mut Option<FileDescriptor> = &mut get_fd_entry(list, fd as usize);
    match fd_entry {
        None => return GENERIC_ERRNO,
        Some(fd) => {
            match fd.kind {
                FileDescriptorType::FILE => try_close_file(fd.inner),
                FileDescriptorType::DIR => unsafe { fs_helpers::close_directory(fd.inner); },
                FileDescriptorType::TCP => todo!(),
                FileDescriptorType::STDIN => try_close_file(fd.inner),
                FileDescriptorType::STDOUT => try_close_file(fd.inner),
                FileDescriptorType::STDERR => try_close_file(fd.inner),
            };
        }
    };

    *fd_entry = None;
    0
}


#[unsafe(no_mangle)]
pub extern "C" fn sys_writev(_fd: i32, _iov: *const u8, _iovcnt: usize) -> isize {
    todo!()
}

fn try_close_file(fd: u64) {
    unsafe { fs_helpers::close_file(fd) };
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_unlink(name: *const i8) -> i32  {
    let result = unsafe { fs_helpers::unlink(name) };
    if result.success {
        0
    } else {
        GENERIC_ERRNO
    }
}
