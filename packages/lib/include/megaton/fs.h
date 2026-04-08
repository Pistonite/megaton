// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#pragma once
#include <cstdint>
#include <stdexcept>
#include <nn/fs.h>


#define BOTWTOOLKIT_TCP_SEND2 // todo: replace with BOTWTOOLKIT_TCP_SEND (double check this)
#ifdef BOTWTOOLKIT_TCP_SEND2
    namespace botw::tcp {
        void sendf(const char* args, ...);
    }
#else
    namespace botw::tcp {    
        void sendf(const char* args, ...) {}
    }
#endif

using FD = std::uint32_t;

// Functions exported to mod developers
namespace megaton { 
    void debug_show_fd_list();
}

struct iovec {
    char   *iov_base;  /* Base address. */
    size_t iov_len;    /* Length. */
};


enum class FileDescriptorType {
    UNUSED,
    FILE,
    TCP,
    DIR,
    STDIN,
    STDOUT,
    STDERR,
};

struct FileDescriptor {
    private:
        FileDescriptorType type; 
        uint64_t internal_fd;
        
    public:
        uint64_t seek_pos; // used for files

        FileDescriptor(FileDescriptorType t, uint64_t internal_fd): type(t), internal_fd(internal_fd), seek_pos(0) { }
        FileDescriptor(FileDescriptorType t): type(t) { }
        FileDescriptor(): type(FileDescriptorType::UNUSED) {  };

        FileDescriptorType get_type() {
            return type;
        }

        uint64_t get_internal_fd() {
            return internal_fd;
        }
};


inline FileDescriptor create_fd_file(uint64_t inner) {
    return FileDescriptor(FileDescriptorType::FILE, inner);
}

inline FileDescriptor create_fd_dir(uint64_t inner) {
    return FileDescriptor(FileDescriptorType::DIR, inner);
}

inline FileDescriptor create_fd_stdin() {
    return FileDescriptor(FileDescriptorType::STDIN);
}

inline FileDescriptor create_fd_stdout() {
    return FileDescriptor(FileDescriptorType::STDOUT);
}

inline FileDescriptor create_fd_stderr() {
    return FileDescriptor(FileDescriptorType::STDERR);
}
