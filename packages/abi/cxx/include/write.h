#pragma once

#include <cstdint>


typedef int64_t isize;
typedef uint64_t usize;

// source: hermit-os/kernel/src/fd/mod.rs
typedef int32_t FileDescriptor;

// source: hermit-os/hermit-rs/hermit-abi/src/lib.rs
struct iovec {
    void* iov_base;
    usize iov_len;
};



extern "C" isize sys_write(FileDescriptor fd, const uint8_t* buf, usize len);

// pub unsafe extern "C" fn sys_writev(fd: FileDescriptor, iov: *const iovec, iovcnt: usize) -> isize 
extern "C" isize sys_writev(FileDescriptor fd, const iovec* iov, usize len);