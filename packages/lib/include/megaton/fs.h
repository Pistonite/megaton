// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#pragma once
#include <cstdint>
#include <stdexcept>
#include <megaton/__priv/nn/fs.h>


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

namespace megaton {
    void debug_show_fd_list();
}

struct iovec {
    char   *iov_base;  /* Base address. */
    size_t iov_len;    /* Length. */
};


enum FileDescriptorType {
    UNUSEDFDT,
    FILEFDT,
    TCPFDT,
    DIRFDT,
    STDINFDT,
    STDOUTFDT,
    STDERRFDT,
};

struct FileFDU {
    uint64_t internalFD;
    uint64_t seek_pos;
};

/* Contains data related to internal state or internal switch identifiers, 
or is otherwise useful for dealing with the type of FD  */ 
union FDU {
    FileFDU FILE;
    uint64_t TCP;
    uint64_t DIR;
    // could collapse these last 4 into 1 union member, since they store the same kind of data. Not sure what it would be called.
    bool STDIN;  
    bool STDOUT;
    bool STDERR;
    bool UNUSED;
};

struct FileDescriptor {
    private:
        FileDescriptorType type; 
        FDU val;  
        
    public:
        FileDescriptor(FileDescriptorType t, FDU v): type(t), val(v) { }
        FileDescriptor(): type(FileDescriptorType::UNUSEDFDT), val(FDU{ .UNUSED = true }) {  };

        FileDescriptorType getType() {
            return type;
        }

        FDU& getVal() {
            return val;
        }
};


inline FileDescriptor create_fd_file(uint64_t inner) {
    FileFDU fdu = { .internalFD=inner, .seek_pos=0 };
    return FileDescriptor { FileDescriptorType::FILEFDT, FDU{ .FILE=fdu} };
}

inline FileDescriptor create_fd_dir(uint64_t inner) {
    return FileDescriptor { FileDescriptorType::DIRFDT, FDU{ .DIR=inner} };
}

inline FileDescriptor create_fd_stdin() {
    return FileDescriptor { FileDescriptorType::STDINFDT, FDU{ .STDIN=true} };
}

inline FileDescriptor create_fd_stdout() {
    return FileDescriptor { FileDescriptorType::STDOUTFDT,  FDU{ .STDOUT=true}  };
}

inline FileDescriptor create_fd_stderr() {
    return FileDescriptor { FileDescriptorType::STDERRFDT,  FDU{ .STDERR=true}  };
}
