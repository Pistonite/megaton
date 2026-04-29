// // SPDX-License-Identifier: GPL-3.0-or-later
// // Copyright (c) 2025-2026 Megaton contributors


#include <megaton/fs.h>


extern "C" NNResult write_file(uint64_t fd, const uint8_t* buf, uint64_t size, size_t position) {
    nn::fs::WriteOption option = nn::fs::WriteOption::CreateOption(nn::fs::WriteOptionFlag_Flush);
    nn::Result result = nn::fs::WriteFile({ fd }, position, (void*) buf, size, option);
    NNResult resultVal = { result.IsSuccess(), result.GetModule(), result.GetDescription() };
    return resultVal;
}

