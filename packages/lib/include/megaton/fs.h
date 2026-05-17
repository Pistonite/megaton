// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#pragma once
#include <cstdint>
#include <nn/fs.h>

// namespace botw::tcp {
//     void sendf(const char* args, ...);
// }

enum class FileDescriptorType {
    FILE,
    DIR,
    TCP,
    STDIN,
    STDOUT,
    STDERR,
};
struct FileDescriptor {
    uint64_t inner;
    FileDescriptorType kind;
    uint64_t seek_offset;
};

struct NNResult {
    bool success;
    int32_t module;
    int32_t description;
};

struct OpenResult {
    NNResult result;
    FileDescriptor fd;
};

struct ReadResult {
    NNResult result;
    size_t bytes_read;
};

struct GetEntryTypeResult { 
    NNResult result;
    nn::fs::DirectoryEntryType entry_type;
};

struct GetSizeResult {
    NNResult result;
    uint64_t size;
};


extern "C" NNResult __megaton_lib_fs_write_file(uint64_t nn_fd, const uint8_t* buf, uint64_t size, size_t position);
extern "C" OpenResult __megaton_lib_fs_open(const char* name, int32_t flags, uint32_t mode);
extern "C" GetEntryTypeResult __megaton_lib_fs_get_entry_type(const char* name);
extern "C" GetSizeResult __megaton_lib_fs_get_file_size(uint64_t nn_fd);
extern "C" NNResult __megaton_lib_fs_unlink(const char* name);
extern "C" void __megaton_lib_fs_close_file(uint64_t nn_fd);
extern "C" void __megaton_lib_fs_close_dir(uint64_t nn_fd);
extern "C" void __megaton_lib_log(const uint8_t* buf, uint64_t len);
extern "C" void __megaton_lib_fs_write_stdout(const uint8_t* buf, uint64_t len);
extern "C" void __megaton_lib_fs_write_stderr(const uint8_t* buf, uint64_t len);


namespace megaton {
    void debug_show_fd_list();
}
