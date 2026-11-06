// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors
// * * * * *
#pragma once

#include <stdint.h>
#include <nn/fs.h>

struct GetEntryTypeResult {
    u32 result;
    nn::fs::DirectoryEntryType value;
};

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus
//
extern void _megakernel_close_directory(uint64_t nn_fd);
extern void _megakernel_close_file(uint64_t nn_fd);
extern GetEntryTypeResult _megakernel_get_entry_type(const int8_t *name);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
