use std::ffi::c_int;



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
    FILE,
    DIR,
    TCP,
    STDIN,
    STDOUT,
    STDERR,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FileDescriptor {
    pub inner: u64,
    pub kind: FileDescriptorType,
    pub seek_offset: u64,
}


#[repr(C)]
pub struct OpenResult {
    pub result: NNResult,
    pub fd: FileDescriptor
}

#[repr(C)]
pub struct ReadResult {
    pub result: NNResult,
    pub bytes_read: usize,
}

#[repr(C)]
pub struct GetEntryTypeResult {
    pub result: NNResult,
    pub entry_type: DirectoryEntryType
}

#[repr(C)]
pub struct GetSizeResult {
    pub result: NNResult,
    pub size: i64
}


#[repr(C)]
pub struct StatResult {
    pub result: NNResult,
    pub stat_val: stat
}


unsafe extern "C" {
    #[link_name = "__megaton_lib_fs_write_file"]
    pub unsafe fn write_file(nn_fd: u64, seek_offset: u64, buf: *const u8, len: usize) -> NNResult;

    #[link_name = "__megaton_lib_fs_open"]
    pub unsafe fn open(name: *const i8,  flags: i32, mode: i32) -> OpenResult;

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
}



// https://github.com/hermit-os/hermit-rs/blob/82146cf059bf3894eea1e96beed9da72b99b9d5a/hermit-abi/src/lib.rs#L44
pub const STDIN_FILENO: usize = 0;
pub const STDOUT_FILENO: usize = 1;
pub const STDERR_FILENO: usize = 2;
pub const O_RDONLY: i32 = 0o0;
pub const O_WRONLY: i32 = 0o1;
pub const O_RDWR: i32 = 0o2;
pub const O_CREAT: i32 = 0o100;
pub const O_EXCL: i32 = 0o200;
pub const O_TRUNC: i32 = 0o1000;
pub const O_APPEND: i32 = 0o2000;
pub const O_NONBLOCK: i32 = 0o4000;
pub const O_DIRECTORY: i32 = 0o200000;

pub const FS_ERR_MODULE: i32 = 2; // all fs errors will have module = 2
pub const PATH_NOT_FOUND: i32      = 1;
pub const PATH_ALREADY_EXISTS: i32 = 2;
pub const TARGET_LOCKED: i32       = 7;
pub const DIRECTORY_NOT_EMPTY: i32 = 8;

// https://github.com/hermit-os/hermit-rs/blob/82146cf059bf3894eea1e96beed9da72b99b9d5a/hermit-abi/src/lib.rs#L169
type time_t = i64;

// https://github.com/hermit-os/hermit-rs/blob/82146cf059bf3894eea1e96beed9da72b99b9d5a/hermit-abi/src/lib.rs#L284
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


// https://github.com/hermit-os/kernel/blob/884cdccf6a5ca532b5aad102a530e2d6e7cf5b25/src/time.rs
/// Represent the number of seconds and nanoseconds since
/// the Epoch (1970-01-01 00:00:00 +0000 (UTC))
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct timespec {
	/// seconds
	pub tv_sec: time_t,
	/// nanoseconds
	pub tv_nsec: i32,
}

impl timespec {
	pub fn from_usec(microseconds: i64) -> Self {
		Self {
			tv_sec: (microseconds / 1_000_000),
			tv_nsec: ((microseconds % 1_000_000) * 1000) as i32,
		}
	}

	pub fn into_usec(&self) -> Option<i64> {
		self.tv_sec
			.checked_mul(1_000_000)
			.and_then(|usec| usec.checked_add((self.tv_nsec / 1000).into()))
	}
}


/* Handle representing an opened file. */
struct FileHandle {
    internal: u64
}

/* Handle representing an opened directory. */
struct DirectoryHandle {
    internal: u64
}

/* Kinds of entries within a directory. */
#[repr(C)]
enum DirectoryEntryType {
    DirectoryEntryType_Directory,
    DirectoryEntryType_File,
}


/* Bitfield to define the kinds of entries to open from a directory. */
#[repr(C)]
enum OpenDirectoryMode {
    OpenDirectoryMode_Directory = 1 << 0,
    OpenDirectoryMode_File = 1 << 1,
    OpenDirectoryMode_All = (1 << 0) | (1 << 1),
}