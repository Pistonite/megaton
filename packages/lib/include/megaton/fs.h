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



using isize = std::int32_t;
using usize = std::uint32_t;
using u64 = std::uint64_t;
using FileDescriptor = std::uint32_t;

namespace megaton {
    using usize = std::uint32_t;
    using u64 = std::uint64_t;
    using FileDescriptor = std::uint32_t;
    nn::fs::FileHandle open_file(const char* path);
    bool file_exists(const char* path);
    bool write_file(nn::fs::FileHandle fd, const char* content);
    void close_file(nn::fs::FileHandle fd);
    void debugShowFDList();
}

struct iovec {
    char   *iov_base;  /* Base address. */
    size_t iov_len;    /* Length. */
};


enum FDType {
    FILEFDT,
    TCPFDT,
    DIRFDT,
    STDINFDT,
    STDOUTFDT,
    STDERRFDT,
    UNUSEDFDT,
};

struct FileFDU {
    u64 internalFD;
    u64 seek_pos;
};

union FDU {
    FileFDU FILE;
    u64 TCP;
    u64 DIR;
    bool STDIN;  // could collapse these last 4 into 1 union member. Not sure what it would be called.
    bool STDOUT;
    bool STDERR;
    bool UNUSED;
};

struct FD {
    private:
        FDType type; 
        FDU val;  // data related to internal state or internal switch identifiers,
        // or is otherwise useful for dealing with the type of FD 
        
    public:
        FD(FDType t, FDU v): type(t), val(v) { }
        FD(): type(FDType::UNUSEDFDT), val(FDU{ .UNUSED = true }) {  };

        FDType getType() {
            return type;
        }

        FDU& getVal() {
            return val;
        }
};


static FD create_fd_file(u64 inner) {
    FileFDU fdu = { .internalFD=inner, .seek_pos=0 };
    return FD { FDType::FILEFDT, FDU{ .FILE=fdu} };
}

static FD create_fd_dir(u64 inner) {
    return FD { FDType::DIRFDT, FDU{ .DIR=inner} };
}

static FD create_fd_stdin() {
    return FD { FDType::STDINFDT, FDU{ .STDIN=true} };
}

static FD create_fd_stdout() {
    return FD { FDType::STDOUTFDT,  FDU{ .STDOUT=true}  };
}

static FD create_fd_stderr() {
    return FD { FDType::STDERRFDT,  FDU{ .STDERR=true}  };
}
