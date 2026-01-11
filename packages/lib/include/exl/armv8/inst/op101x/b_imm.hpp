// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/op101x.hpp>

namespace exl::armv8::inst::impl::op101x {

struct InsnBranchImm : public InsnOp101x {

    static constexpr u8 Op0 = 0b000;

    ACCESSOR(Op, 31);
    ACCESSOR(Imm26, 0, 26);

    enum Op {
        B = 0,
        BL = 1,
    };

    constexpr InsnBranchImm(Op op, u32 relative_address) : InsnOp101x(Op0) {
        SetOp(op);
        SetImm26(relative_address / 4);
    }
};
} // namespace exl::armv8::inst::impl::op101x
