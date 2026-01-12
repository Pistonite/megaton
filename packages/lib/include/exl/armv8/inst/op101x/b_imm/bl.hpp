// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/op101x/b_imm.hpp>

namespace exl::armv8::inst {

struct BranchLink : public impl::op101x::InsnBranchImm {

    constexpr BranchLink(u32 relative_address)
        : InsnBranchImm(InsnBranchImm::BL, relative_address) {}
};

static_assert(BranchLink(0x4440).Value() == 0x94001110, "");
static_assert(BranchLink(0x4200).Value() == 0x94001080, "");
static_assert(BranchLink(0x6900).Value() == 0x94001A40, "");
static_assert(BranchLink(0x0008).Value() == 0x94000002, "");
} // namespace exl::armv8::inst
