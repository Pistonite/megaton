// // SPDX-License-Identifier: GPL-3.0-or-later
// // Copyright (c) 2025-2026 Megaton contributors


#include <megaton/fs.h>


// fills out NNResult struct based on data from nn::Result.
// (nn::Result is not FFI-safe)
NNResult buildSimpleResult(nn::Result result) { 
    return { result.IsSuccess(), result.GetModule(), result.GetDescription() };
}

extern "C" NNResult __megaton_lib_fs_write_file(uint64_t nn_fd, const uint8_t* buf, uint64_t size, size_t position) {
    nn::fs::WriteOption option = nn::fs::WriteOption::CreateOption(nn::fs::WriteOptionFlag_Flush);
    nn::Result result = nn::fs::WriteFile({ nn_fd }, position, (void*) buf, size, option);
    return buildSimpleResult(result);
}

uint32_t hermit_to_nn_flags(uint32_t hermit_open_option_flags) {
    // hermit OpenOption defintion: https://github.com/hermit-os/kernel/blob/ec0fc3572c9d8deba725df8b7eb000980034a9f6/src/fd/mod.rs#L53
    // nnheaders OpenOption definition: https://github.com/open-ead/nnheaders/blob/0547381a6166ea54fb306a53a02683a8527475fd/include/nn/fs/fs_types.h#L51

    uint32_t nn_flags = 0;
    int access = hermit_open_option_flags & 0b11;
    if (access == 0) nn_flags |= nn::fs::OpenMode_Read;
    else if (access == 1) nn_flags |= nn::fs::OpenMode_Write;
    else if (access == 2) nn_flags |= nn::fs::OpenMode_ReadWrite;
    if (access != 0) nn_flags |= nn::fs::OpenMode_Append;
    return nn_flags;
}

extern "C" OpenResult __megaton_lib_fs_open(const char* name, int32_t flags, uint32_t mode) {
    const int O_CREAT  = 0100;  // create file if it doesn't exist
    // const int O_EXCL   = 0200;  // fail if file already exists
    const int O_TRUNC  = 01000; // truncate file to zero length on open
    const int O_APPEND = 02000; // set initial seek position to end of file
    botw::tcp::sendf("Calling sys_open with %s %d %u!\n", name, flags, mode);

    bool o_creat  = flags & O_CREAT;
    // bool o_excl   = flags & O_EXCL;
    bool o_trunc  = flags & O_TRUNC;
    bool o_append = flags & O_APPEND;

    OpenResult open_result;
    nn::Result result;
    nn::fs::DirectoryEntryType opened_type;
    result = nn::fs::GetEntryType(&opened_type, name);
    open_result.result = buildSimpleResult(result);
    open_result.fd = { .inner=9999, .kind=FileDescriptorType::FILE, .seek_offset=0 };

    if (result.IsFailure()) { // no such file or directory
        if (!o_creat) {    
            return open_result;
        }
        botw::tcp::sendf("File %s does not exist: creating!\n", name);
        result = nn::fs::CreateFile(name, 0);
        open_result.result = buildSimpleResult(result);
        if (result.IsFailure()) {
            botw::tcp::sendf("Creating file failed!\n");
            return open_result;
        }
        opened_type = nn::fs::DirectoryEntryType_File;
    }

    if (opened_type == nn::fs::DirectoryEntryType_File) { 
        botw::tcp::sendf("sys_open target %s is a file!\n", name);
        uint32_t open_mode = hermit_to_nn_flags(flags);

        nn::fs::FileHandle inner;
        result = nn::fs::OpenFile(&inner, name, open_mode);
        open_result.result = buildSimpleResult(result);
        if (result.IsFailure()) return open_result;
        
        open_result.fd.kind = FileDescriptorType::FILE;
        open_result.fd.inner = inner._internal;
    
        if (o_trunc) {
            result = nn::fs::SetFileSize(inner, 0);
            open_result.result = buildSimpleResult(result);
            if(result.IsFailure()) return open_result;    
        }

        if (o_append) {
            long file_size = 0;
            result = nn::fs::GetFileSize(&file_size, inner);
            open_result.result = buildSimpleResult(result);
            if(result.IsFailure()) return open_result;
            open_result.fd.seek_offset = file_size;
        }

        return open_result;
    } else {
        open_result.fd.kind = FileDescriptorType::DIR;
        
        nn::fs::DirectoryHandle inner;
        result = nn::fs::OpenDirectory(&inner, name, nn::fs::OpenDirectoryMode_All);
        open_result.result = buildSimpleResult(result);
        if(result.IsFailure()) return open_result;
        open_result.fd.inner = inner._internal;
    }
    return open_result;
}

extern "C" ReadResult __megaton_lib_fs_read_file(uint64_t nn_fd, uint64_t seek_pos, char* buf, uint64_t len) {
    uint64_t bytes_read;
    //                                  u64 *outSize, nn::fs::FileHandle handle, s64 offset, void *buffer, u64 bufferSize
    nn::Result result = nn::fs::ReadFile(&bytes_read, { ._internal=nn_fd, }, seek_pos, buf, len);
    ReadResult read_result = {  .result=buildSimpleResult(result), .bytes_read=bytes_read };
    return read_result;
}

extern "C" GetEntryTypeResult __megaton_lib_fs_get_entry_type(const char* name) {
    nn::fs::DirectoryEntryType type;
    nn::Result result = nn::fs::GetEntryType(&type, name);
    // TODO: Am I being paranoid here? What actually happens to type if this function returns a failure?
    // entry_type defaults to file on failure to ensure the field always has valid data
    GetEntryTypeResult entry_type_result = {  .result=buildSimpleResult(result), .entry_type=nn::fs::DirectoryEntryType_File };
    if(result.IsSuccess()) entry_type_result.entry_type = type; 
    return entry_type_result;
}

extern "C" GetSizeResult __megaton_lib_fs_get_file_size(uint64_t nn_fd) {
    long size;
    nn::Result result = nn::fs::GetFileSize(&size, { nn_fd });
    // size defaults to 0 on failure to ensure the field always has valid data
    GetSizeResult get_size_result = { .result=buildSimpleResult(result), .size=0  };
    if(result.IsSuccess()) get_size_result.size = size;
    return get_size_result;
}

extern "C" void __megaton_lib_fs_close_file(uint64_t nn_fd) {
    nn::fs::CloseFile( { nn_fd });
}

extern "C" void __megaton_lib_fs_close_dir(uint64_t nn_fd) {
    nn::fs::CloseDirectory( { nn_fd });
}

extern "C" NNResult __megaton_lib_fs_unlink(const char* name) {
    nn::fs::DirectoryEntryType type;
    nn::Result result = nn::fs::GetEntryType(&type, name);
    if(result.IsFailure()) return buildSimpleResult(result);
     
    switch(type) {
        case nn::fs::DirectoryEntryType_File: {
            result = nn::fs::DeleteFile(name);
            return buildSimpleResult(result);
        }
        case nn::fs::DirectoryEntryType_Directory: {
            result = nn::fs::DeleteDirectory(name);
            return buildSimpleResult(result);
        }
        default: {}
    }

    return buildSimpleResult(result);
}

namespace megaton {
    void debug_show_fd_list() {
        // do nothing
    }

}