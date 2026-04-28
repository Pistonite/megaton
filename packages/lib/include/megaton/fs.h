// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025-2026 Megaton contributors

#pragma once
#include <cstdint>
#include <nn/fs.h>

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
