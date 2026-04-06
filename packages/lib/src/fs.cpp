// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#include <megaton/fs.h>
#include <string.h>


using isize = std::int32_t;
using usize = std::uint32_t;
using u64 = std::uint64_t;
using FileDescriptor = std::uint32_t;



const int NUM_FDS = 1000;
static FD FDList[NUM_FDS] = { FD() };
static uint32_t current_umask = 0022;


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



namespace impl {
    // internal helper functions go here
    nn::fs::FileHandle getFileHandle(FDU fileFDU) {
        nn::fs::FileHandle fh = nn::fs::FileHandle { };
        fh._internal = fileFDU.FILE.internalFD;
        return fh;
    }

    void write_file(FD fd, const char* buf, usize len){
        FileFDU fdu = fd.getVal().FILE;
        nn::fs::FileHandle fh = impl::getFileHandle(fd.getVal());
        nn::fs::WriteOption options = nn::fs::WriteOption::CreateOption(0);

        nn::Result result = nn::fs::WriteFile(fh, fdu.seek_pos, buf, len, options);
        fdu.seek_pos += len;
        if(result.IsFailure()) {
            botw::tcp::sendf("sys_write: nn::fs::WriteFile failed! Exit code=%d", result.GetInnerValueForDebug());
        }
    }
    
    

    int32_t stat_file(nn::fs::FileHandle handle, const char* name, FileAttr* stat) {
        nn::Result result;
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

    int32_t stat_dir(nn::fs::DirectoryHandle handle, const char* name, FileAttr* stat) {
        nn::Result result;
        // TODO: What values should these have?
        stat->st_gid = 0;
        stat->st_dev = 0;
        stat->st_ino = 0;
        stat->st_nlink = 0;

        u32 S_IRUSR = 0400;
        u32 S_IWUSR = 0200;
        stat->st_mode = S_IRUSR | S_IWUSR;

        long file_size = 0;
        stat->st_size = file_size;
        int block_size = 1024;
        stat->st_blksize = block_size;
        if(file_size % block_size == 0) stat->st_blocks = file_size / block_size;
        else stat->st_blocks = (file_size / block_size) + 1;

        timespec timestamp;
        timestamp.tv_sec = 0;
        timestamp.tv_nsec = 0;

        stat->st_atim = timestamp;
        stat->st_mtim = timestamp;
        stat->st_ctim = timestamp;

        return 0;
    } 
}


// Syscall Definitions

extern "C" FileDescriptor sys_open(const char* name, int32_t flags, uint32_t mode) {
    botw::tcp::sendf("Library: sys_open called! name=%s flags=%d mode=%u\n", name, flags, mode);
    // e.g. sys_open called name=sd:/testfile3.txt flags=577 mode=511
    nn::Result result;
    
    nn::fs::DirectoryEntryType type;
    result = nn::fs::GetEntryType(&type, name);
    if(result.IsFailure()) { 
        if(mode & 0100) { // if file doesn't exist, and create flag given by caller, try to create the file
            result = nn::fs::CreateFile(name, 0);
            if(result.IsFailure()) {
                botw::tcp::sendf("Library sys_open: nn::fs::CreateFile failed! Exit code=%d\n", result.GetInnerValueForDebug());
                return -1;
            }
            type = nn::fs::DirectoryEntryType_File;
        }
    }

    // could probably be condensed via polymorphism
    FileDescriptor outerFD = 999;
    if(type == nn::fs::DirectoryEntryType_File) {
        nn::fs::FileHandle inner;
        result = nn::fs::OpenFile(&inner, name, nn::fs::OpenMode_ReadWrite | nn::fs::OpenMode_Append); // todo: What to do if failure occurs?
        if(result.IsFailure()) {
            botw::tcp::sendf("Library sys_open: nn::fs::OpenFile failed! Exit code=%d\n", result.GetInnerValueForDebug());
            return -1;
        }
        FD fd = create_fd_file(inner._internal);
        outerFD = insertIntoFDList(fd);
        botw::tcp::sendf("\tsys_open: successfully returning file fd=%d\n", outerFD);
    }
    else { // type == nn::fs::DirectoryEntryType_Directory
        nn::fs::DirectoryHandle inner;
        result = nn::fs::OpenDirectory(&inner, name, nn::fs::OpenDirectoryMode_All);
        FD fd = create_fd_dir(inner._internal);
        
        outerFD = insertIntoFDList(fd);
        botw::tcp::sendf("\tsys_open: successfully returning directory fd=%d\n", outerFD);
    }
    
    return outerFD;
}


extern "C" void sys_write(FileDescriptor fd, const char* buf, usize len) {
    FD innerFd = FDList[fd];
    botw::tcp::sendf("Library: sys_write called! fd=%d buf=%s len=%u\n", fd, buf, len);
    switch(innerFd.getType()) {
        case FDType::FILEFDT: {
            impl::write_file(innerFd, buf, len);
            break;
        }
        default: {
            botw::tcp::sendf("Library: sys_write called for invalid type with enum val %d", innerFd.getType());
        }
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
            nn::fs::FileHandle fh = impl::getFileHandle(outerFD.getVal());
            nn::fs::FlushFile(fh);
            nn::fs::CloseFile(fh);
            removeFromFDList(fd);
            return 0;
        }
        case FDType::DIRFDT: {
            nn::fs::DirectoryHandle dh = nn::fs::DirectoryHandle { };
            dh._internal = outerFD.getVal().DIR;
            nn::fs::CloseDirectory(dh);
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
            nn::fs::FileHandle fh = impl::getFileHandle(outerFD.getVal());
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
            botw::tcp::sendf("Library: sys_read called for unsupported FDType: %d! fd=%d", outerFD.getType(), fd);
            return -1;
        }
    }

    
}

namespace megaton {

    void debugShowFDList() {
        char msg[NUM_FDS*12] = {0};
        int msgIndex = 0;
        //   0=>42 1=>43 2=>73

        for (int i = 0; i < NUM_FDS; i++)
        {
            FD fd = FDList[i];
            switch(fd.getType()) {
                case FDType::FILEFDT: {
                    msgIndex += snprintf(msg + msgIndex, 14, "%d=>%ld ", i, fd.getVal().FILE.internalFD);
                    break;
                }
                case FDType::DIRFDT: {
                    msgIndex += snprintf(msg + msgIndex, 14, "%d=>%ld ", i, fd.getVal().DIR);
                    break;
                }
                case FDType::TCPFDT: {
                    msgIndex += snprintf(msg + msgIndex, 14, "%d=>%ld ", i, fd.getVal().TCP);
                    break;
                }
                case FDType::STDINFDT: {
                    msgIndex += snprintf(msg + msgIndex, 14, "%d=>stdin ", i);
                    break;
                }
                case FDType::STDOUTFDT: {
                    msgIndex += snprintf(msg + msgIndex, 14, "%d=>stdout ", i);
                    break;
                }
                case FDType::STDERRFDT: {
                    msgIndex += snprintf(msg + msgIndex, 14, "%d=>stderr ", i);
                    break;
                }
                default: {
                    break;
                }
            }
        }
        botw::tcp::sendf("%s", msg);
    }
}

// Used here: https://github.com/hermit-os/rust/blob/042a556d8d0247361ed97e5d9217bb477c487be3/library/std/src/sys/fs/hermit.rs#L585
extern "C" int32_t sys_stat(const char* name, FileAttr* stat) {
    botw::tcp::sendf("sys_stat called! filename: %s\n",name);

    nn::fs::DirectoryEntryType type;
    nn::Result result = nn::fs::GetEntryType(&type, name);
    if (result.IsFailure()) return -1;
    
    if(type == nn::fs::DirectoryEntryType_File) {
        nn::fs::FileHandle handle;
        nn::Result result = nn::fs::OpenFile(&handle, name, nn::fs::OpenMode_Read);
        if(result.IsFailure()) return -1;
        int32_t stat_result = impl::stat_file(handle, name, stat);
        nn::fs::CloseFile(handle);
        return stat_result;
    } else if(type == nn::fs::DirectoryEntryType_Directory) {
        nn::fs::DirectoryHandle handle;
        nn::Result result = nn::fs::OpenDirectory(&handle, name, nn::fs::OpenMode_Read);
        if(result.IsFailure()) return -1;
        int32_t stat_result = impl::stat_dir(handle, name, stat);
        nn::fs::CloseDirectory(handle);
        return stat_result;
    }
    return -1;
}

extern "C" int32_t sys_mkdir(const char* name, uint32_t mode) {
    botw::tcp::sendf("Library: sys_mkdir called! name=%s mode=%u\n", name, mode);
    nn::Result result = nn::fs::CreateDirectory(name);
    if(result.IsFailure()) {
        botw::tcp::sendf("Library: sys_mkdir failed! Exit code=%d\n", result.GetInnerValueForDebug());
        return -1;
    }
    botw::tcp::sendf("Library: sys_mkdir success!\n");
    return 0;
}

extern "C" FileDescriptor sys_opendir(const char* name) {
    botw::tcp::sendf("Library: sys_opendir called! name=%s\n", name);
    nn::fs::DirectoryHandle inner;
    nn::Result result = nn::fs::OpenDirectory(&inner, name, nn::fs::OpenDirectoryMode_All);
    if(result.IsFailure()) {
        botw::tcp::sendf("Library: sys_opendir failed! Exit code=%d\n", result.GetInnerValueForDebug());
        return -1;
    }
    FD fd = create_fd_dir(inner._internal);
    FileDescriptor outerFD = insertIntoFDList(fd);
    botw::tcp::sendf("Library: sys_opendir success! fd=%d\n", outerFD);
    return outerFD;
}

