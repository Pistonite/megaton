// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/op101x.hpp>

namespace exl::armv8::inst::impl::op101x {

struct InsnBranchRegister : public InsnOp101x {

    static constexpr u8 Op0 = 0b110;
    static constexpr u32 Op1 = 0b10000000000000;

    ACCESSOR(Opc, 21, 25);
    ACCESSOR(UBROp2, 16, 21);
    ACCESSOR(Op3, 10, 16);
    ACCESSOR(Rn, 5, 10);
    ACCESSOR(Op4, 0, 5);

    constexpr InsnBranchRegister(u8 opc, u8 op2) : InsnOp101x(Op0) {
        SetOp1(Op1);
        SetOpc(opc);
        SetUBROp2(op2);
    }
};
} // namespace exl::armv8::inst::impl::op101x
