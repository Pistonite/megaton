#pragma once
#include <cstdint>
#include <stdexcept>
#include <megaton/__priv/nn/fs.h>



using isize = std::int32_t;
using usize = std::uint32_t;
using u64 = std::uint64_t;
using FileDescriptor = std::uint32_t;

#define FOO2 // todo: replace with BOTWTOOLKIT_TCP_SEND (double check this)
#ifdef FOO2
    namespace botw::tcp {
        void sendf(const char* args, ...);
    }
#else
    namespace botw::tcp {    
        void sendf(const char* args, ...) {}
    }
#endif

namespace megaton {
    using usize = std::uint32_t;
    using u64 = std::uint64_t;
    using FileDescriptor = std::uint32_t;
    nn::fs::FileHandle open_file(const char* path);
    bool write_file(nn::fs::FileHandle fd, const char* content);
    void close_file(nn::fs::FileHandle fd);
}

struct iovec {
    char   *iov_base;  /* Base address. */
    size_t iov_len;    /* Length. */
};
