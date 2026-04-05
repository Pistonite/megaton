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

struct FD {
    private:
        FDType type;
        u64 val;
        
    public:
        FD(FDType t, u64 v): type(t), val(v) { }
        FD(): type(FDType::UNUSEDFDT), val(420) { };

        FDType getType() {
            return type;
        }

        u64 getInner() {
            return val;
        }
};
