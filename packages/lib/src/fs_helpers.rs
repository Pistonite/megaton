use std::ffi::c_int;

#[derive(Clone, Copy, Debug)]
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
    pub error_code: i32,   // denotes error, or 0 on success
    pub fd: FileDescriptor
}

unsafe extern "C" {
    // include!("toolkit/tcp.hpp");
    // #[namespace = "botw::tcp"]
    #[link_name = "foobar"]
    pub unsafe fn write_file(nn_fd: u64, seek_offset: u64, buf: *const u8, len: usize) -> isize;

    #[link_name = "foobar"]
    pub unsafe fn open(name: *const i8,  flags: i32, mode: i32) -> OpenResult;
}



// https://github.com/hermit-os/hermit-rs/blob/82146cf059bf3894eea1e96beed9da72b99b9d5a/hermit-abi/src/lib.rs#L44
pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;
pub const O_RDONLY: i32 = 0o0;
pub const O_WRONLY: i32 = 0o1;
pub const O_RDWR: i32 = 0o2;
pub const O_CREAT: i32 = 0o100;
pub const O_EXCL: i32 = 0o200;
pub const O_TRUNC: i32 = 0o1000;
pub const O_APPEND: i32 = 0o2000;
pub const O_NONBLOCK: i32 = 0o4000;
pub const O_DIRECTORY: i32 = 0o200000;

// https://github.com/hermit-os/hermit-rs/blob/82146cf059bf3894eea1e96beed9da72b99b9d5a/hermit-abi/src/lib.rs#L169
type time_t = i64;

// https://github.com/hermit-os/kernel/blob/884cdccf6a5ca532b5aad102a530e2d6e7cf5b25/src/fs/mod.rs#L288
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct FileAttr {
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
enum DirectoryEntryType {
    DirectoryEntryType_Directory,
    DirectoryEntryType_File,
}


/* Bitfield to define the kinds of entries to open from a directory. */
enum OpenDirectoryMode {
    OpenDirectoryMode_Directory = 1 << 0,
    OpenDirectoryMode_File = 1 << 1,
    OpenDirectoryMode_All = (1 << 0) | (1 << 1),
}