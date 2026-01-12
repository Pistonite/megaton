// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/op100x.hpp>

namespace exl::armv8::inst::impl::op100x {

struct InsnPcRelAddr : public InsnOp100x {

    static constexpr u8 Op0 = 0b000;

    ACCESSOR(Op, 31);
    ACCESSOR(Immlo, 29, 31);
    ACCESSOR(Immhi, 5, 24);
    ACCESSOR(Rd, 0, 5);

    enum Op : u8 {
        Op_ADR = 0,
        Op_ADRP = 1,
    };

    constexpr InsnPcRelAddr(reg::Register reg, u32 imm, Op op)
        : InsnOp100x(Op0) {
        /*
            static_assert(reg.Is64());
        */
        SetOp(op);
        SetImmlo(imm);
        SetImmhi(imm >> ImmloMask.Count);
        SetRd(reg.Index());
    }
};
}; // namespace exl::armv8::inst::impl::op100x
