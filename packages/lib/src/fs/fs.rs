use std::{ffi::CString, sync::{Mutex, OnceLock}};

const MIN_FD: usize = 100; // fds other than stdio will be returned as their index into the list + MIN_FD
const NUM_FDS: usize = 1000;
const GENERIC_ERRNO: i32 = -1;
// TODO: Allow user to configure stdio paths
const STDIN_PATH: &[u8] = b"sd:/megaton_stdin.txt\0";
const STDOUT_PATH: &[u8] = b"sd:/megaton_stdout.txt\0";
const STDERR_PATH: &[u8] = b"sd:/megaton_stderr.txt\0";
const LOG_PATH: &[u8] = b"sd:/megaton_logs.txt\0";

use crate::fs::fs_helpers::{self, FileDescriptor, FileDescriptorType, GetEntryTypeResult, NNResult, O_CREAT, O_RDONLY, O_RDWR, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, write_stderr, write_stdout};

static LIST: Mutex<[Option<FileDescriptor>; NUM_FDS]> = Mutex::new([const { None }; NUM_FDS]);

fn insert_into_fd_list(fd: FileDescriptor) -> Option<usize> {
    let mut list = LIST.try_lock().unwrap();
    for i in 0..NUM_FDS {
        if list[i].is_none() {
            list[i] = Some(fd);
            return Some(i + MIN_FD);
        }
    }
    None
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_show_fd_list(){
    let list = &mut LIST.try_lock().unwrap();
    for i in 0..NUM_FDS {
        if let Some(fd) = &list[i] {
            megaton_log(format!("{}=>(type={:?} inner={} seek={})\t", i+MIN_FD, fd.kind, fd.inner, fd.seek_offset).as_str());
            // unsafe { sendf(b"%d=>{type=? inner=%u}\t\0".as_ptr() as *const std::ffi::c_char, fd.inner) };
        }
    }
}

fn get_fd_entry(fd: usize) -> Option<FileDescriptor> {
    if fd < MIN_FD || fd - MIN_FD >= NUM_FDS {
        return None
    }

    let list = LIST.try_lock().unwrap();
    list[fd-MIN_FD].clone()
}

fn set_fd_entry(fd: usize, fd_entry: Option<FileDescriptor>) {
    assert!(fd >= MIN_FD && fd - MIN_FD < NUM_FDS);
    LIST.try_lock().unwrap()[fd-MIN_FD] = fd_entry;
}

#[allow(dead_code)]
pub fn try_initialize_stdio(){
    // unsafe { fs_helpers::try_init_stdio() };
}

fn megaton_log(msg: &str) {
    try_initialize_stdio();
    let len = (&msg).len();
    let msg = CString::new(msg).unwrap();
    unsafe { fs_helpers::megaton_log(msg.as_c_str().as_ptr() as *const u8, len as u64) };
}

fn log_error(msg: &str, result: &NNResult) {
    megaton_log(format!("Megaton Error: NNResult description: {}\t message: {}\n", result.description, msg).as_str());
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_open(name: *const i8, flags: i32, mode: i32) -> i32 {
    megaton_log(format!("sys_open called with name={:#?} flags={} mode={}\n", 
        unsafe { std::ffi::CStr::from_ptr(name as *const std::ffi::c_char) }, flags, mode
    ).as_str());
    let open_result = unsafe { fs_helpers::open(name, flags, mode) };
    if open_result.result.success  {
        match insert_into_fd_list(open_result.fd) {
            Some(fd_index) => fd_index as i32,
            None => GENERIC_ERRNO,
        }
    } else {
        log_error("Megaton: sys_open failed!\n", &open_result.result);
        GENERIC_ERRNO
    }
}

#[unsafe(no_mangle)]
pub fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize {
    megaton_log(format!("sys_write called with fd={} len={}\n", fd, len).as_str());
    match fd as usize {
        STDIN_FILENO => return GENERIC_ERRNO as isize,
        STDOUT_FILENO => {
            unsafe { write_stdout(buf, len as u64) };
            return len as isize;
        },
        STDERR_FILENO => {
            unsafe { write_stderr(buf, len as u64) };
            return len as isize;
        },
        _ => {}
    };
    
    let fd_index = fd as usize;
    let fd_entry= get_fd_entry(fd_index);
    match fd_entry {
        None => GENERIC_ERRNO as isize,
        Some(fd) => {
            match fd.kind {
                FileDescriptorType::FILE => try_write_file(fd, fd_index, buf, len),
                FileDescriptorType::DIR => GENERIC_ERRNO as isize,
                FileDescriptorType::TCP => todo!(),
                
            }
        }
    }
}


fn try_write_file(mut fd: FileDescriptor, fd_index: usize, buf: *const u8, len: usize) -> isize {
    // unsafe { sendf(b"try_write_file %d %s\n\0".as_ptr() as *const std::ffi::c_char, fd.inner, buf) };
    let write_result = unsafe { fs_helpers::write_file(fd.inner, buf,  len, fd.seek_offset) };
    
    if write_result.success {
        fd.seek_offset += len as u64;
        set_fd_entry(fd_index, Some(fd));
        len as isize
    } else {
        log_error( "Megaton: sys_write failed!", &write_result);
        match write_result.description {
            _ => GENERIC_ERRNO as isize // TODO: Map nn error to errno
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_read(fd_index: i32, buf: *mut u8, len: usize) -> isize {
    // Todo: Hold lock for the entire duration of read/write 
    megaton_log(format!("sys_read called with fd={} len={}\n", fd_index,len).as_str());

    match fd_index as usize {
        STDIN_FILENO => return GENERIC_ERRNO as isize,
        STDOUT_FILENO | STDERR_FILENO => {
            todo!()
        }
        _ => {}
    };

    let fd_entry: Option<FileDescriptor> = get_fd_entry(fd_index as usize);
    match fd_entry {
        None => -1,
        Some(fd) => {
            match fd.kind {
                FileDescriptorType::FILE => try_read_file(fd, fd_index as usize, buf, len),
                FileDescriptorType::DIR => todo!(),
                FileDescriptorType::TCP => todo!(),
            }
        }
    }
}

fn try_read_file(mut fd_entry: FileDescriptor, fd_index: usize, buf: *mut u8, len: usize) -> isize {
    let read_result = unsafe { fs_helpers::read_file(fd_entry.inner, fd_entry.seek_offset, buf, len as u64) };
    if !read_result.result.success {
        // unsafe { sendf(b"Result was module=%d description=%d\n\0".as_ptr() as *const std::ffi::c_char, read_result.result.module, read_result.result.description) }
        // assert!(read_result.result.module == fs_helpers::FS_ERR_MODULE);
        return GENERIC_ERRNO as isize
    } 
    fd_entry.seek_offset += read_result.bytes_read as u64;
    set_fd_entry(fd_index, Some(fd_entry));

    read_result.bytes_read as isize
}

#[unsafe(no_mangle)]
pub extern "C" fn sys_fstat(fd: i32, stat: *mut fs_helpers::stat) -> i32 {
    megaton_log( format!("sys_fstat called for fd={}\n", fd).as_str());
    match fd as usize {
        STDIN_FILENO | STDOUT_FILENO | STDERR_FILENO => {
            return GENERIC_ERRNO;
        }
        _ => {}
    };

    if let Some(fd) = get_fd_entry(fd as usize) {
        stat_file(&fd, stat)
    } else {
        return GENERIC_ERRNO
    }
}

// https://github.com/hermit-os/hermit-rs/blob/111a7b480a18ce1b6c576d9dac02a688203432ee/hermit/src/syscall/mod.rs#L187
#[unsafe(no_mangle)]
pub extern "C" fn sys_stat(name: *const i8, stat: *mut fs_helpers::stat) -> i32 {
    megaton_log(format!("sys_stat called with name={:?}\n", 
        unsafe { std::ffi::CStr::from_ptr(name as *const std::ffi::c_char) }
    ).as_str());
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
        match open_result.result.description {
            fs_helpers::TARGET_LOCKED => {
                already_open = true;
            },
            _ => {
                log_error("sys_stat: open file failed!", &open_result.result);
                return GENERIC_ERRNO;
            }
        }
    }

    let handle: FileDescriptor = if already_open {
        megaton_log("sys_stat failed! TODO: Get fd entry with matching name");
        todo!() // get fd entry with matching name
    } else {
        open_result.fd
    };

    let result = stat_file(&handle, stat);
    try_close_file(handle.inner);
    result
}

fn stat_file(fd: &FileDescriptor, stat: *mut fs_helpers::stat) -> i32 {
    let mut stat = unsafe { *stat };
    stat.st_uid = 0; // owner user id 
    stat.st_gid = 0; // owner group id
    stat.st_dev = 0; // device number
    stat.st_ino = 0; // inode number
    stat.st_nlink = 0; // link count

    const S_IRUSR: u32 = 0o400; // read
    const S_IWUSR: u32 = 0o200; // write
    const S_ISREG: u32 = 0o100000; // regular file
    const S_ISDIR: u32 = 0o040000; // directory
    const S_ISSOCK: u32 = 0140000; // socket

    let mut st_mode: u32 = S_IRUSR | S_IWUSR;
    match fd.kind {
        FileDescriptorType::FILE => st_mode |= S_ISREG,
        FileDescriptorType::DIR => st_mode |= S_ISDIR,
        FileDescriptorType::TCP => st_mode |= S_ISSOCK,
    };
    stat.st_mode = st_mode;

    let size_result = unsafe { fs_helpers::get_file_size(fd.inner) };
    if !size_result.result.success {
        return GENERIC_ERRNO;
    } else {
        stat.st_size = size_result.size;
        stat.st_blksize = 1000;
        stat.st_blocks = ((size_result.size as f64) / 1000.0f64).ceil() as i64;
        return 0;
    }
}


#[unsafe(no_mangle)]
pub fn sys_close(fd: i32) -> i32 {
    megaton_log(format!("Megaton: Closing file {}!\n", fd).as_str());
    let fd_entry  =  get_fd_entry(fd as usize);

    match fd as usize {
        STDIN_FILENO | STDOUT_FILENO | STDERR_FILENO => {
            todo!();
        }
        _ => {}
    };

    match fd_entry {
        None => {
            megaton_log(format!("Error: No entry for fd {} exists!\n", fd).as_str());
            return GENERIC_ERRNO;
        }
        Some(fd) => {
            match fd.kind {
                FileDescriptorType::FILE => try_close_file(fd.inner),
                FileDescriptorType::DIR => unsafe { fs_helpers::close_directory(fd.inner); },
                FileDescriptorType::TCP => todo!()
            };
        }
    };

    set_fd_entry(fd as usize, None);
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
    megaton_log(format!("sys_unlink called with name={:?}\n", unsafe { std::ffi::CStr::from_ptr(name as *const std::ffi::c_char) }).as_str());
    let result = unsafe { fs_helpers::unlink(name) };
    if result.success {
        0
    } else {
        GENERIC_ERRNO
    }
}
