#include <megaton/fs.h>
#include <string.h>


using isize = std::int32_t;
using usize = std::uint32_t;
using u64 = std::uint64_t;
using FileDescriptor = std::uint32_t;

static FD create_fd_file(u64 inner) {
    return FD { FDType::FILEFDT, inner };
}

// static FD create_fd_tcp(u64 inner) {
//     return FD { FDType::TCPFDT, FDU{ .TCPFDU = inner } };
// }

// static FD create_fd_dir(u64 inner) {
//     return FD { FDType::DIRFDT, FDU{ .DIRFDU = inner } };
// }

static FD create_fd_stdin() {
    return FD { FDType::STDINFDT, 69 };
}

static FD create_fd_stdout() {
    return FD { FDType::STDOUTFDT, 70 };
}

static FD create_fd_stderr() {
    return FD { FDType::STDERRFDT, 71 };
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
    botw::tcp::sendf("Unable to allocate FD - FDList is full!\n");
    return 0;
}

void removeFromFDList(FileDescriptor fd) {
    FDList[fd] = FD();
}


uint32_t hermit_to_nn_flags(uint32_t hermit_open_option_flags) {
    // hermit OpenOption defintion: https://github.com/hermit-os/kernel/blob/ec0fc3572c9d8deba725df8b7eb000980034a9f6/src/fd/mod.rs#L53
    // nnheaders OpenOption definition: https://github.com/open-ead/nnheaders/blob/0547381a6166ea54fb306a53a02683a8527475fd/include/nn/fs/fs_types.h#L51
    return hermit_open_option_flags;
}

extern "C" FileDescriptor sys_open(const char* name, int32_t flags, uint32_t mode) {
    nn::fs::FileHandle inner;
    botw::tcp::sendf("Library: sys_open called! name=%s flags=%d mode=%u\n", name, flags, mode);
    // e.g. sys_open called name=sd:/testfile3.txt flags=577 mode=511
    // create=true

    // todo:check if file exists before calling create
    // i think nn::fs::CreateFile will error if the file exists
    
    
    
    nn::Result result;
    if(mode & 0100) {
        if(!megaton::file_exists(name)) {
            result = nn::fs::CreateFile(name, 0);
            if(result.IsFailure()) {
                botw::tcp::sendf("Library sys_open: nn::fs::CreateFile failed! Exit code=%d\n", result.GetInnerValueForDebug());
            }
        }
    }

    result = nn::fs::OpenFile(&inner, name, nn::fs::OpenMode_ReadWrite | nn::fs::OpenMode_Append); // todo: What to do if failure occurs?
    if(result.IsFailure()) {
        botw::tcp::sendf("Library sys_open: nn::fs::OpenFile failed! Exit code=%d\n", result.GetInnerValueForDebug());
        return -1;
    }
    FD fd = create_fd_file(inner._internal);
    FileDescriptor outerFD = insertIntoFDList(fd);
    botw::tcp::sendf("\tsys_open: successfully returning fd=%d\n", outerFD);
    return outerFD;
}


extern "C" void sys_write(FileDescriptor fd, const char* buf, usize len) {
    u64 innerFd = FDList[fd].getInner();
    botw::tcp::sendf("Library: sys_write called! fd=%d buf=%s len=%u inner=%ld\n", fd, buf, len, innerFd);
    struct nn::fs::FileHandle inner = { innerFd };
    nn::fs::WriteOption options = nn::fs::WriteOption::CreateOption(nn::fs::WriteOptionFlag_Flush);
    
    nn::Result result = nn::fs::WriteFile(inner, 0, buf, strlen(buf), options);
    if(result.IsFailure()) {
        botw::tcp::sendf("sys_write: nn::fs::WriteFile failed! Exit code=%d", result.GetInnerValueForDebug());
    }
}

extern "C" isize sys_writev(FileDescriptor fd, const iovec* iov, usize iovcnt) {
    botw::tcp::sendf("Library: sys_writev called! fd=%d\n", fd);
    return 0;
}

extern "C" int32_t sys_close(FileDescriptor fd) {
    botw::tcp::sendf("Library: sys_close called! fd=%d\n", fd);

    FD outerFD = FDList[fd];
    switch(outerFD.getType()) {
        case FDType::FILEFDT: {
            nn::fs::FileHandle fh = nn::fs::FileHandle { };
            fh._internal = outerFD.getInner();
            nn::fs::CloseFile(fh);
            removeFromFDList(fd);
            return 0;
        }
        default: {
            return 0;
        }
    }
}


extern "C" isize sys_read(FileDescriptor fd, u8* buf, usize len) {
    botw::tcp::sendf("Library: sys_read called! fd=%d len=%d\n", fd, len);
    FD outerFD = FDList[fd];
    switch(outerFD.getType()) {
        case FDType::FILEFDT: {
            nn::fs::FileHandle fh = nn::fs::FileHandle { };
            fh._internal = outerFD.getInner();
            size_t bytes_read = 0;
            nn::Result result = nn::fs::ReadFile(&bytes_read, fh, 0, buf, len);
            if(result.IsFailure()) {
                botw::tcp::sendf("sys_read: nn::fs::ReadFile failed! Exit code=%d", result.GetInnerValueForDebug());
                return -1;
            }
            botw::tcp::sendf("Library: sys_read success! bytes_read=%d", bytes_read);
            return (isize)bytes_read;
                
            
        }
        default: {
            botw::tcp::sendf("sys_read called for unsupported FDType: %d! fd=%d", outerFD.getType(), fd);
            return -1;
        }
    }

    
}

namespace megaton {
    // If debugShowFDList doesn't work, try this first!
    // void debugShowFDListSafe(){
    
    //}

    void debugShowFDList() {
        char msg[NUM_FDS*12] = {0};
        int msgIndex = 0;
        //   0=>42 1=>43 2=>73

        for (int i = 0; i < NUM_FDS; i++)
        {
            FD fd = FDList[i];
            if(fd.getType() != FDType::UNUSEDFDT) {
                msgIndex += snprintf(msg + msgIndex, 14, "%d=>%ld ", i, fd.getInner());
            }
        }
        botw::tcp::sendf("%s", msg);
    }

    
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

    bool file_exists(const char* path) {
        
        nn::fs::DirectoryEntryType type;
        nn::Result result = nn::fs::GetEntryType(&type, path);

        if (result.IsFailure()) {
            // botw::tcp::sendf("file_exists nn::fs::GetEntryType failed! path=%s error_code=%d\n", path, result.GetInnerValueForDebug());
            return false;
        }
        bool exists = type != nn::fs::DirectoryEntryType_Directory;
        botw::tcp::sendf("file %s exists=%d \n", path, exists);

        return exists;
    }
}

struct timespec {
    long tv_sec;
    long tv_nsec;
};

// see: https://github.com/hermit-os/kernel/blob/884cdccf6a5ca532b5aad102a530e2d6e7cf5b25/src/fs/mod.rs#L288
struct FileAttr {
	u64 st_dev;
	u64 st_ino;
	u64 st_nlink;
	/// access permissions
	u32 st_mode; // see: https://github.com/hermit-os/kernel/blob/ec0fc3572c9d8deba725df8b7eb000980034a9f6/src/fd/mod.rs#L164
	/// user id
	u32 st_uid;
	/// group id
	u32 st_gid;
	/// device id
	u64 st_rdev;
	/// size in bytes
	int64_t st_size;
	/// block size
	int64_t st_blksize;
	/// size in blocks
	int64_t st_blocks;
	/// time of last access
	timespec st_atim;
	/// time of last modification
	timespec st_mtim;
	/// time of last status change
	timespec st_ctim;
};


// It is used here: https://github.com/hermit-os/rust/blob/042a556d8d0247361ed97e5d9217bb477c487be3/library/std/src/sys/fs/hermit.rs#L585
extern "C" int32_t sys_stat(const char* name, FileAttr* stat) {
    botw::tcp::sendf("sys_stat called! filename: %s\n",name);
    if(!megaton::file_exists(name)) {
        return -1;
    }
    nn::fs::FileHandle handle;
    nn::Result result = nn::fs::OpenFile(&handle, name, nn::fs::OpenMode_Read);
    if(result.IsFailure()) return -1;

    // TODO: What values should these have?
    stat->st_gid = 0;
    stat->st_dev = 0;
    stat->st_ino = 0;
    stat->st_nlink = 0;

    u32 S_IRUSR = 0400;
    u32 S_IWUSR = 0200;
    stat->st_mode = S_IRUSR | S_IWUSR;


    long file_size = 0;
    result = nn::fs::GetFileSize(&file_size, handle);
    if(result.IsFailure()) return -1;

    stat->st_size = file_size;
    int block_size = 1024;
    stat->st_blksize = block_size;
    if(file_size % block_size == 0) stat->st_blocks = file_size / block_size;
    else stat->st_blocks = (file_size / block_size) + 1;


    nn::fs::FileTimeStamp mtime;
    result = nn::fs::GetFileTimeStampForDebug(&mtime, name);
    if(result.IsFailure()) return -1;

    timespec timestamp;
    timestamp.tv_sec = mtime.mTime1;
    timestamp.tv_nsec = 0;

    stat->st_atim = timestamp;
    stat->st_mtim = timestamp;
    stat->st_ctim = timestamp;

    return 0;
}



namespace impl {
    // internal helper functions go here
}
