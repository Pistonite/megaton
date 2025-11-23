#pragma once

#include <cstdint>
#include "rust/cxx.h"


typedef int64_t isize;
typedef uint64_t usize;

// source: hermit-os/kernel/src/fd/mod.rs
typedef int32_t FileDescriptor;

// source: hermit-os/hermit-rs/hermit-abi/src/lib.rs
struct iovec {
    void* iov_base;
    usize iov_len;
};

namespace fs {
    uint64_t open_file(rust::Str path, int flags, uint32_t mode);
    uint64_t open_dir(rust::Str path);
    uint64_t write_file(uint64_t fd, int64_t pos, uint8_t* buf, usize len);
    void close_file(uint64_t fd);
}

extern "C" isize sys_write(FileDescriptor fd, const uint8_t* buf, usize len);
// 
// pub unsafe extern "C" fn sys_writev(fd: FileDescriptor, iov: *const iovec, iovcnt: usize) -> isize 
// extern "C" isize sys_writev(FileDescriptor fd, const iovec* iov, usize len);