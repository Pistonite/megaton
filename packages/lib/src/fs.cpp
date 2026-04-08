// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#include <megaton/fs.h>
#include <string.h>

const int NUM_FDS = 1000;
static FileDescriptor FileDescriptorList[NUM_FDS];
static uint32_t current_umask = 0022;


void init_stdio() {
    FileDescriptor fd_stdin = create_fd_stdin();
    FileDescriptor fd_stdout = create_fd_stdout();
    FileDescriptor fd_stderr = create_fd_stderr();

    FileDescriptorList[0] = fd_stdin;
    FileDescriptorList[1] = fd_stdout;
    FileDescriptorList[2] = fd_stderr;
}

struct timespec {
    long tv_sec;
    long tv_nsec;
};

// see: https://github.com/hermit-os/kernel/blob/884cdccf6a5ca532b5aad102a530e2d6e7cf5b25/src/fs/mod.rs#L288
struct FileAttr {
	uint64_t st_dev;
	uint64_t st_ino;
	uint64_t st_nlink;
	/// access permissions
	uint32_t st_mode; // see: https://github.com/hermit-os/kernel/blob/ec0fc3572c9d8deba725df8b7eb000980034a9f6/src/fd/mod.rs#L164
	/// user id
	uint32_t st_uid;
	/// group id
	uint32_t st_gid;
	/// device id
	uint64_t st_rdev;
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
    uint32_t hermit_to_nn_flags(uint32_t hermit_open_option_flags) {
        // hermit OpenOption defintion: https://github.com/hermit-os/kernel/blob/ec0fc3572c9d8deba725df8b7eb000980034a9f6/src/fd/mod.rs#L53
        // nnheaders OpenOption definition: https://github.com/open-ead/nnheaders/blob/0547381a6166ea54fb306a53a02683a8527475fd/include/nn/fs/fs_types.h#L51
        return hermit_open_option_flags;
    }

    // internal helper functions go here
    nn::fs::FileHandle get_file_handle(FileDescriptor fd) {
        return { fd.get_internal_fd() };
    }
    
    FD insert_into_fd_list(FileDescriptor fd) {
        for(FD i = 3; i < NUM_FDS; i++) {
            if(FileDescriptorList[i].get_type() == FileDescriptorType::UNUSED) {
                FileDescriptorList[i] = fd;
                return i;
            }
        }
        botw::tcp::sendf("Unable to allocate FD - FileDescriptorList is full!\n");
        return 0;
    }

    void remove_from_fd_list(FD fd) {
        FileDescriptorList[fd] = FileDescriptor();
    }

    void write_file(FileDescriptor fd, const char* buf, uint64_t len){
        nn::fs::FileHandle fh = impl::get_file_handle(fd);
        nn::fs::WriteOption options = nn::fs::WriteOption::CreateOption(0);

        nn::Result result = nn::fs::WriteFile(fh, fd.seek_pos, buf, len, options);
        fd.seek_pos += len;
        if(result.IsFailure()) {
            botw::tcp::sendf("sys_write: nn::fs::WriteFile failed! Exit code=%d\n", result.GetInnerValueForDebug());
        }
    }
    
    int32_t stat_file(nn::fs::FileHandle handle, const char* name, FileAttr* stat) {
        nn::Result result;
        // TODO: What values should these have?
        stat->st_gid = 0;
        stat->st_dev = 0;
        stat->st_ino = 0;
        stat->st_nlink = 0;

        uint32_t S_IRUSR = 0400;
        uint32_t S_IWUSR = 0200;
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

        uint32_t S_IRUSR = 0400;
        uint32_t S_IWUSR = 0200;
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

extern "C" FD sys_open(const char* name, int32_t flags, uint32_t mode) {
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
    FD outer_fd = 999;
    if(type == nn::fs::DirectoryEntryType_File) {
        nn::fs::FileHandle inner;
        result = nn::fs::OpenFile(&inner, name, nn::fs::OpenMode_ReadWrite | nn::fs::OpenMode_Append); // todo: What to do if failure occurs?
        if(result.IsFailure()) {
            botw::tcp::sendf("Library sys_open: nn::fs::OpenFile failed! Exit code=%d\n", result.GetInnerValueForDebug());
            return -1;
        }
        FileDescriptor fd = create_fd_file(inner._internal);
        outer_fd = impl::insert_into_fd_list(fd);
        botw::tcp::sendf("\tsys_open: successfully returning file fd=%d\n", outer_fd);
    }
    else { // type == nn::fs::DirectoryEntryType_Directory
        nn::fs::DirectoryHandle inner;
        result = nn::fs::OpenDirectory(&inner, name, nn::fs::OpenDirectoryMode_All);
        FileDescriptor fd = create_fd_dir(inner._internal);
        
        outer_fd = impl::insert_into_fd_list(fd);
        botw::tcp::sendf("\tsys_open: successfully returning directory fd=%d\n", outer_fd);
    }
    
    return outer_fd;
}


extern "C" void sys_write(FD fd, const char* buf, uint64_t len) {
    FileDescriptor inner_fd = FileDescriptorList[fd];
    botw::tcp::sendf("Library: sys_write called! fd=%d buf=%s len=%u\n", fd, buf, len);
    switch(inner_fd.get_type()) {
        case FileDescriptorType::FILE: {
            impl::write_file(inner_fd, buf, len);
            break;
        }
        default: {
            botw::tcp::sendf("Library: sys_write called for invalid type with enum val %d\n", inner_fd.get_type());
        }
    }
    
}

extern "C" int64_t sys_writev(FD fd, const iovec* iov, uint64_t iovcnt) {
    botw::tcp::sendf("Library: sys_writev called! fd=%d\n", fd);
    return 0;
}

extern "C" int32_t sys_close(FD fd) {
    botw::tcp::sendf("Library: sys_close called! fd=%d\n", fd);

    FileDescriptor outer_fd = FileDescriptorList[fd];
    switch(outer_fd.get_type()) {
        case FileDescriptorType::FILE: {
            nn::fs::FileHandle fh = impl::get_file_handle(outer_fd);
            nn::fs::FlushFile(fh);
            nn::fs::CloseFile(fh);
            impl::remove_from_fd_list(fd);
            return 0;
        }
        case FileDescriptorType::DIR: {
            nn::fs::DirectoryHandle dh = nn::fs::DirectoryHandle { outer_fd.get_internal_fd() };
            nn::fs::CloseDirectory(dh);
            impl::remove_from_fd_list(fd);
            return 0;
        }
        default: {
            return 0;
        }
    }
}

extern "C" int64_t sys_read(FD fd, u8* buf, uint64_t len) {
    botw::tcp::sendf("Library: sys_read called! fd=%d len=%d\n", fd, len);
    if(fd < 0 || fd >= NUM_FDS) {
        botw::tcp::sendf("Library: Error in sys_read: FD %d is out of bounds!\n", fd);
        return -ENOENT;
    }

    FileDescriptor outer_fd = FileDescriptorList[fd];
    switch(outer_fd.get_type()) {
        case FileDescriptorType::FILE: {
            nn::fs::FileHandle fh = impl::get_file_handle(outer_fd);
            size_t bytes_read = 0;
            nn::Result result = nn::fs::ReadFile(&bytes_read, fh, outer_fd.seek_pos, buf, len);
            if(result.IsFailure()) {
                botw::tcp::sendf("sys_read: nn::fs::ReadFile failed! Exit code=%d\n", result.GetInnerValueForDebug());
                return -EIO;
            }
            botw::tcp::sendf("Library: sys_read success! bytes_read=%d\n", bytes_read);
            outer_fd.seek_pos += bytes_read;
            return (int64_t)bytes_read;
        }
        default: {
            botw::tcp::sendf("Library: sys_read called for unsupported FDType: %d! fd=%d\n", outer_fd.get_type(), fd);
            return -ENOENT;
        }
    }    
}

extern "C" int32_t sys_unlink(const char* name) {
    nn::fs::DirectoryEntryType type;
    botw::tcp::sendf("Library: sys_unlink called for %s\n", name);

    nn::Result result = nn::fs::GetEntryType(&type, name);
    if(result.IsFailure()){
        botw::tcp::sendf("Library: sys_unlink: nn::fs::GetEntryType failed with %d!\n", result.GetInnerValueForDebug());
        return -ENOENT; // todo : investigate if there are proper/more meaningful error codes
    } 
    switch(type) {
        case nn::fs::DirectoryEntryType_File: {
            botw::tcp::sendf("Library: Attempting to delete file %s\n", name);
            result = nn::fs::DeleteFile(name);
            if(result.IsFailure()){
                botw::tcp::sendf("Library: sys_unlink: nn::fs::DeleteFile failed with %d!\n", result.GetInnerValueForDebug());
                return -1;
            }
            break;
        }
        case nn::fs::DirectoryEntryType_Directory: {
            botw::tcp::sendf("Library: Attempting to delete directory %s\n", name);
            result = nn::fs::DeleteDirectory(name);
            if(result.IsFailure()){
                botw::tcp::sendf("Library: sys_unlink: nn::fs::DeleteDirectory failed with %d!\n", result.GetInnerValueForDebug());
                return -1;
            }
            break;
        }
        default: {
            botw::tcp::sendf("Library: Invalid DirectoryEntryType: %d\n", type);
            return -1;
        }
    }

    if(result.IsFailure()) return -1;
    else return 0;
} 


// Used here: https://github.com/hermit-os/rust/blob/042a556d8d0247361ed97e5d9217bb477c487be3/library/std/src/sys/fs/hermit.rs#L585
extern "C" int32_t sys_stat(const char* name, FileAttr* stat) {
    botw::tcp::sendf("sys_stat called! filename: %s\n",name);

    nn::fs::DirectoryEntryType type;
    nn::Result result = nn::fs::GetEntryType(&type, name);
    if (result.IsFailure()) {
        botw::tcp::sendf("library sys_stat: nn::fs::GetEntryType returned Failure result\n");
        return -1;
    }
    
    if(type == nn::fs::DirectoryEntryType_File) {
        nn::fs::FileHandle handle;
        nn::Result result = nn::fs::OpenFile(&handle, name, nn::fs::OpenMode_Read);
        if(result.IsFailure()){
            botw::tcp::sendf("Library: sys_stat: nn::fs::OpenFile failed with %d!\n", result.GetInnerValueForDebug());
            return -1;
        }
        int32_t stat_result = impl::stat_file(handle, name, stat);
        nn::fs::CloseFile(handle);
        return stat_result;
    } else if(type == nn::fs::DirectoryEntryType_Directory) {
        nn::fs::DirectoryHandle handle;
        nn::Result result = nn::fs::OpenDirectory(&handle, name, nn::fs::OpenMode_Read);
        if(result.IsFailure()){
            botw::tcp::sendf("Library: sys_stat: nn::fs::OpenDirectory failed with %d!\n", result.GetInnerValueForDebug());

            return -1;
        }
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

extern "C" FD sys_opendir(const char* name) {
    botw::tcp::sendf("Library: sys_opendir called! name=%s\n", name);
    nn::fs::DirectoryHandle inner;
    nn::Result result = nn::fs::OpenDirectory(&inner, name, nn::fs::OpenDirectoryMode_All);
    if(result.IsFailure()) {
        botw::tcp::sendf("Library: sys_opendir failed! Exit code=%d\n", result.GetInnerValueForDebug());
        return -1;
    }
    FileDescriptor fd = create_fd_dir(inner._internal);
    FD outer_fd = impl::insert_into_fd_list(fd);
    botw::tcp::sendf("Library: sys_opendir success! fd=%d\n", outer_fd);
    return outer_fd;
}


namespace megaton {

    void debug_show_fd_list(){
        botw::tcp::sendf("Library: Printing FileDescriptorList!\n");

        for (int i = 0; i < NUM_FDS; i++)
        {
            FileDescriptor fd = FileDescriptorList[i];

            switch(fd.get_type()) {
                case FileDescriptorType::FILE: { 
                    botw::tcp::sendf("%d=>file (%ld) (seek=%ld) ", i, fd.get_internal_fd(), fd.seek_pos);
                    break;
                }
                case FileDescriptorType::DIR: {
                    botw::tcp::sendf("%d=>dir (%ld) ", i, fd.get_internal_fd());
                    break;
                }
                case FileDescriptorType::TCP: {
                    botw::tcp::sendf("%d=>tcp (%ld) ", i, fd.get_internal_fd());
                    break;
                }
                case FileDescriptorType::STDIN: {
                    botw::tcp::sendf("%d=>stdin ", i);
                    break;
                }
                case FileDescriptorType::STDOUT: {
                    botw::tcp::sendf("%d=>stdout ", i);
                    break;
                }
                case FileDescriptorType::STDERR: {
                    botw::tcp::sendf("%d=>stderr ", i);
                    break;
                }
                default: {
                    break;
                }
            }
        }
        botw::tcp::sendf("\nLibrary: Printing FileDescriptorList Done!\n");
    }
}
