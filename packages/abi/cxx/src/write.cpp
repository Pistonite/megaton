
#include "write.h"
// #include <stdint.h>
// pub unsafe extern "C" fn sys_write(fd: FileDescriptor, buf: *const u8, len: usize) -> isize

// extern "C" isize sys_write(FileDescriptor fd, const uint8_t* buf, usize len) {}

extern "C" isize sys_write(FileDescriptor fd, const uint8_t* buf, usize len) {
    return 0;
}

extern "C" isize sys_writev(FileDescriptor fd, const iovec* iov, usize len) {
    return 0;
}