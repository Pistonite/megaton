#include <cstdint>
#include <stdexcept>
#include <toolkit/io/file.hpp>
#ifdef BOTW_TCP_DEBUG
    #include <toolkit/tcp.hpp>
#else
    namespace botw{
        namespace tcp {
            void sendf(const char* args...) {}
        }
    }
#endif

using usize = std::uint32_t;
using u64 = std::uint64_t;
using FileDescriptor = std::uint32_t;

enum FDType {
    FILE,
    TCP,
    DIR,
    STDIN,
    STDOUT,
    STDERR,
    UNUSED,
};

union FDU {
    u64 FILE;
    u64 TCP;
    u64 DIR;
    bool STDIN;
    bool STDOUT;
    bool STDERR;
    bool UNUSED;
};

struct FD {
    private:
        FDType type;
        FDU val;
        FD(FDType t, FDU v): type(t), val(v) { }

    public:
        FD(): type(FDType::UNUSED), val(FDU{ .UNUSED = true }) {  };

        FDType getType() {
            return type;
        }

        FDU getInner() {
            return val;
        }

        static FD file(nn::fs::FileHandle inner) {
            return FD { FDType::FILE, FDU{ .FILE = inner._internal } };
        }

        static FD tcp(u64 inner) {
            return FD { FDType::TCP, FDU{ .TCP = inner } };
        }

        static FD dir(u64 inner) {
            return FD { FDType::DIR, FDU{ .DIR = inner } };
        }

        static FD stdin() {
            return FD { FDType::STDIN, FDU{ .STDIN = true } };
        }

        static FD stdout() {
            return FD { FDType::STDOUT, FDU{ .STDOUT = true } };
        }

        static FD stderr() {
            return FD { FDType::STDERR, FDU{ .STDERR = true } };
        }
        
        static FD unused(){
            return FD { FDType::UNUSED, FDU{ .UNUSED = true } };
        }
};


void init_stdio() {
    FD stdin = FD::stdin();
    FD stdout = FD::stdout();
    FD stderr = FD::stderr();
    FDList[0] = stdin;
    FDList[1] = stdout;
    FDList[2] = stderr;
    
}

const int NUM_FDS = 1000;
static FD FDList[NUM_FDS] = { FD() };
static char log_buffer[1000] = {};
static FD log_fd = { FDType::FILE, FDU { .FILE = 0 }};



FileDescriptor insertIntoFDList(FD fd) {
    for(FileDescriptor i = 3; i < NUM_FDS; i++) {
        if(FDList[i].getType() == FDType::UNUSED) {
            FDList[i] = fd;
            return i;
        }
    }
    throw std::logic_error("Unable to allocate FD - FDList is full!");
    return 0;
}


uint32_t hermit_to_nn_flags(uint32_t hermit_open_option_flags) {
    // hermit OpenOption defintion: https://github.com/hermit-os/kernel/blob/ec0fc3572c9d8deba725df8b7eb000980034a9f6/src/fd/mod.rs#L53
    // nnheaders OpenOption definition: https://github.com/open-ead/nnheaders/blob/0547381a6166ea54fb306a53a02683a8527475fd/include/nn/fs/fs_types.h#L51
    return hermit_open_option_flags;
}

extern "C" FileDescriptor sys_open(const char* name, int32_t flags, uint32_t mode) {
    nn::fs::FileHandle inner;
    botw::tcp::sendf("Library: sys_open called! name=%s flags=%d mode=%u", name, flags, mode);
    
    nn::Result result = nn::fs::OpenFile(&inner, name, nn::fs::OpenMode_ReadWrite | nn::fs::OpenMode_Append); // todo: What to do if failure occurs?
    if(result.IsFailure()) {
        botw::tcp::sendf("Library: nn::fs::OpenFile failed! Exit code=%d", result.GetInnerValueForDebug());
    }
    FD fd = FD::file(inner);
    FileDescriptor outerFD = insertIntoFDList(fd);
    return outerFD;
}

extern "C" void sys_write(FileDescriptor fd, const char* buf, usize len) {
    u64 innerFd = FDList[fd].getInner().FILE;
    botw::tcp::sendf("Library: sys_write called! fd=%d (inner=%d) buf=%s len=%u", fd, innerFd, buf, len);
    struct nn::fs::FileHandle inner = { innerFd };
    
    nn::Result result = nn::fs::WriteFile(inner, 0, msg, strlen(msg), nn::fs::WriteOption::CreateOption(nn::fs::WriteOptionFlag_Flush));
    if(result.IsFailure()) {
        botw::tcp::sendf("Library: nn::fs::WriteFile failed! Exit code=%d", result.GetInnerValueForDebug());
    }
}



namespace impl {
    #include <string.h>

    #include <nn/fs/fs_directories.h>
    #include <nn/fs/fs_files.h>
    #include <nn/types.h>

    // char* nullTerminateRustStr(rust::Str s) {
    //     // add null terminator to path
    //     char res[50] = {0};
    //     strcpy(res, s);
    //     res[s.length()] = 0;
    // }
    
    

    uint64_t open_dir(const char* path) {
        nn::Result r;
        nn::fs::DirectoryHandle handle;

        r = nn::fs::OpenDirectory(&handle, path, nn::fs::OpenDirectoryMode_All);

        if(r.IsFailure()){
            botw::tcp::sendf("Opening root directory failed!\n");  
            return -1;
        }

        s64 dirCount;
        r = nn::fs::GetDirectoryEntryCount(&dirCount, handle);

        if(r.IsFailure()){
            botw::tcp::sendf("Opening get dir entry count failed!\n");  
            return -1;
        } 
        botw::tcp::sendf("Calling open directory succeeded! Handle: %d. #Entries in dir: %d\n", handle._internal, dirCount);  
        return handle._internal;
    }

    void close_dir(uint64_t fd) {
        botw::tcp::sendf("Calling close_dir!\n");
        struct nn::fs::DirectoryHandle  f = { fd };
        nn::fs::CloseDirectory(f);  
    }

    uint64_t open_file(const char *path, int flags, uint32_t mode) {
        botw::tcp::sendf("Calling open_file!\n");
        
        struct nn::fs::FileHandle handleOut;
        nn::Result result = nn::fs::OpenFile(&handleOut, path, mode); 
        if(result.IsFailure()) {
            botw::tcp::sendf("Calling open_file failed with %d. Description: %d\n", result.GetInnerValueForDebug(), result.GetDescription());
            return -1;
        }
    
        return handleOut._internal;
    }

    void close_file(uint64_t fd) {
        botw::tcp::sendf("Calling close_file!\n");
        struct nn::fs::FileHandle  f = { fd };
        nn::fs::CloseFile(f);   
    }

    uint64_t write_file(uint64_t fd, int64_t pos, uint8_t* buf, usize len) {
        botw::tcp::sendf("Calling close_file!\n");
        struct nn::fs::FileHandle f = { fd };

        const struct nn::fs::WriteOption opt = {0};
        nn::Result result = nn::fs::WriteFile(f, pos, buf, len, opt);
        if(result.IsFailure()) {
            botw::tcp::sendf("Calling write_file failed with %d. Description: %d\n", result.GetInnerValueForDebug(), result.GetDescription());
            return -1; // TODO: Real error codes
        }

        return len;
    }

}
