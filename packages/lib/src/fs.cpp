#include <megaton/fs.h>
#include <string.h>


using usize = std::uint32_t;
using u64 = std::uint64_t;
using FileDescriptor = std::uint32_t;

namespace megaton {

    nn::fs::FileHandle open_file(const char* path) {
        nn::fs::FileHandle f;
        nn::fs::DirectoryEntryType type;
        nn::Result result = nn::fs::GetEntryType(&type, path);

        if (result.IsFailure()) {
            botw::tcp::sendf("File %s does not exist: creating!\n", path);
            result = nn::fs::CreateFile(path, 0);
            if(result.IsFailure()) {
                botw::tcp::sendf("ERROR! Failed to create file %s!\n", path);
            } else {
                botw::tcp::sendf("Created file %s successfully!", path);
            }
        }

        
        result = nn::fs::OpenFile(&f, path, nn::fs::OpenMode_ReadWrite);
        if(result.IsFailure()) {
            botw::tcp::sendf("Failed to open file %s. Returned fd is %d\n", path, f._internal);
        } else {
            botw::tcp::sendf("Opened file %s successfully! Returned fd is %d\n", path, f._internal);
        }

        return f;
    }

    
    bool write_file(nn::fs::FileHandle fd, const char* content) {
        nn::Result result = nn::fs::WriteFile(fd, 0, content, strlen(content), nn::fs::WriteOption::CreateOption(nn::fs::WriteOptionFlag_Flush));
        if(result.IsFailure()) {
            botw::tcp::sendf("Failed to write to file %d\n", fd._internal);
        } else {
            botw::tcp::sendf("Wrote to file %d successfully!\n", fd._internal);
        }
        return result.IsSuccess();
    }

    void close_file(nn::fs::FileHandle fd) {
        nn::fs::CloseFile(fd);
    }
}


enum FDType {
    FILEFDT,
    TCPFDT,
    DIRFDT,
    STDINFDT,
    STDOUTFDT,
    STDERRFDT,
    UNUSEDFDT,
};

union FDU {
    u64 FILEFDU;
    u64 TCPFDU;
    u64 DIRFDU;
    bool STDINFDU;
    bool STDOUTFDU;
    bool STDERRFDU;
    bool UNUSEDFDU;
};

struct FD {
    private:
        FDType type;
        FDU val;
        
    public:
        FD(FDType t, FDU v): type(t), val(v) { }
        FD(): type(FDType::UNUSEDFDT), val(FDU{ .UNUSEDFDU = true }) {  };

        FDType getType() {
            return type;
        }

        FDU getInner() {
            return val;
        }
};

static FD create_fd_file(u64 inner) {
    return FD { FDType::FILEFDT, FDU{ .FILEFDU = inner } };
}

// static FD create_fd_tcp(u64 inner) {
//     return FD { FDType::TCPFDT, FDU{ .TCPFDU = inner } };
// }

// static FD create_fd_dir(u64 inner) {
//     return FD { FDType::DIRFDT, FDU{ .DIRFDU = inner } };
// }

static FD create_fd_stdin() {
    return FD { FDType::STDINFDT, FDU{ .STDINFDU = true } };
}

static FD create_fd_stdout() {
    return FD { FDType::STDOUTFDT, FDU{ .STDOUTFDU = true } };
}

static FD create_fd_stderr() {
    return FD { FDType::STDERRFDT, FDU{ .STDERRFDU = true } };
}

// static FD create_fd_unused(){
//     return FD { FDType::UNUSEDFDT, FDU{ .UNUSEDFDU = true } };
// }


const int NUM_FDS = 1000;
static FD FDList[NUM_FDS] = { FD() };
// static char log_buffer[1000] = {};

void init_stdio() {
    FD fd_stdin = create_fd_stdin();
    FD fd_stdout = create_fd_stdout();
    FD fd_stderr = create_fd_stderr();
    FDList[0] = fd_stdin;
    FDList[1] = fd_stdout;
    FDList[2] = fd_stderr;
}


FileDescriptor insertIntoFDList(FD fd) {
    for(FileDescriptor i = 3; i < NUM_FDS; i++) {
        if(FDList[i].getType() == FDType::UNUSEDFDT) {
            FDList[i] = fd;
            return i;
        }
    }
    botw::tcp::sendf("Unable to allocate FD - FDList is full!");
    return 0;
}


uint32_t hermit_to_nn_flags(uint32_t hermit_open_option_flags) {
    // hermit OpenOption defintion: https://github.com/hermit-os/kernel/blob/ec0fc3572c9d8deba725df8b7eb000980034a9f6/src/fd/mod.rs#L53
    // nnheaders OpenOption definition: https://github.com/open-ead/nnheaders/blob/0547381a6166ea54fb306a53a02683a8527475fd/include/nn/fs/fs_types.h#L51
    return hermit_open_option_flags;
}

extern "C" FileDescriptor sys_open(const char* name, int32_t flags, uint32_t mode) {
    nn::fs::FileHandle inner;
    botw::tcp::sendf("Library: sys_open called by %s! name=%s flags=%d mode=%u", __builtin_FUNCTION(), name, flags, mode);
    
    nn::Result result = nn::fs::OpenFile(&inner, name, nn::fs::OpenMode_ReadWrite | nn::fs::OpenMode_Append); // todo: What to do if failure occurs?
    if(result.IsFailure()) {
        botw::tcp::sendf("Library: nn::fs::OpenFile failed! Exit code=%d", result.GetInnerValueForDebug());
    }
    FD fd = create_fd_file(inner._internal);
    FileDescriptor outerFD = insertIntoFDList(fd);
    return outerFD;
}



extern "C" void sys_write(FileDescriptor fd, const char* buf, usize len) {
    // u64 innerFd = FDList[fd].getInner().FILE;
    botw::tcp::sendf("Library: sys_write called by %s! fd=%d buf=%s len=%u", __builtin_FUNCTION(), fd, buf, len);
    // struct nn::fs::FileHandle inner = { innerFd };
    
    // nn::Result result = nn::fs::WriteFile(inner, 0, msg, strlen(msg), nn::fs::WriteOption::CreateOption(nn::fs::WriteOptionFlag_Flush));
    // if(result.IsFailure()) {
    //     botw::tcp::sendf("Library: nn::fs::WriteFile failed! Exit code=%d", result.GetInnerValueForDebug());
    // }
}

extern "C" int32_t sys_close(FileDescriptor fd) {
    botw::tcp::sendf("Library: sys_close called by %s! fd=%d", __builtin_FUNCTION(), fd);
    return 0;
}

namespace impl {
    #include <string.h>

    // #include <nn/fs/fs_directories.h>
    // #include <nn/fs/fs_files.h>
    // #include <nn/types.h>

    // char* nullTerminateRustStr(rust::Str s) {
    //     // add null terminator to path
    //     char res[50] = {0};
    //     strcpy(res, s);
    //     res[s.length()] = 0;
    // }
    
    

    // uint64_t open_dir(const char* path) {
    //     nn::Result r;
    //     nn::fs::DirectoryHandle handle;

    //     r = nn::fs::OpenDirectory(&handle, path, nn::fs::OpenDirectoryMode_All);

    //     if(r.IsFailure()){
    //         botw::tcp::sendf("Opening root directory failed!\n");  
    //         return -1;
    //     }

    //     s64 dirCount;
    //     r = nn::fs::GetDirectoryEntryCount(&dirCount, handle);

    //     if(r.IsFailure()){
    //         botw::tcp::sendf("Opening get dir entry count failed!\n");  
    //         return -1;
    //     } 
    //     botw::tcp::sendf("Calling open directory succeeded! Handle: %d. #Entries in dir: %d\n", handle._internal, dirCount);  
    //     return handle._internal;
    // }

    // void close_dir(uint64_t fd) {
    //     botw::tcp::sendf("Calling close_dir!\n");
    //     struct nn::fs::DirectoryHandle  f = { fd };
    //     nn::fs::CloseDirectory(f);  
    // }

    // uint64_t open_file(const char *path, int flags, uint32_t mode) {
    //     botw::tcp::sendf("Calling open_file!\n");
        
    //     struct nn::fs::FileHandle handleOut;
    //     nn::Result result = nn::fs::OpenFile(&handleOut, path, mode); 
    //     if(result.IsFailure()) {
    //         botw::tcp::sendf("Calling open_file failed with %d. Description: %d\n", result.GetInnerValueForDebug(), result.GetDescription());
    //         return -1;
    //     }
    
    //     return handleOut._internal;
    // }

    // void close_file(uint64_t fd) {
    //     botw::tcp::sendf("Calling close_file!\n");
    //     struct nn::fs::FileHandle  f = { fd };
    //     nn::fs::CloseFile(f);   
    // }

    // uint64_t write_file(uint64_t fd, int64_t pos, uint8_t* buf, usize len) {
    //     botw::tcp::sendf("Calling close_file!\n");
    //     struct nn::fs::FileHandle f = { fd };

    //     const struct nn::fs::WriteOption opt = {0};
    //     nn::Result result = nn::fs::WriteFile(f, pos, buf, len, opt);
    //     if(result.IsFailure()) {
    //         botw::tcp::sendf("Calling write_file failed with %d. Description: %d\n", result.GetInnerValueForDebug(), result.GetDescription());
    //         return -1; // TODO: Real error codes
    //     }

    //     return len;
    // }

}
