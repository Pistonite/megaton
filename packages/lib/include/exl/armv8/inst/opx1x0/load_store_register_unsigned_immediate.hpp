// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/opx1x0.hpp>
#include <exl/armv8/register.hpp>
#include <exl/armv8/util/sign_extend.hpp>

namespace exl::armv8::inst::impl::opx1x0 {

struct InsnLoadStoreRegisterUnsignedImmediate : public InsnOpx1x0 {

    static constexpr u8 Op0 = 0b0011;
    static constexpr u8 Op2 = 0b10;

    ACCESSOR(Size, 30, 32);
    ACCESSOR(V, 26);
    ACCESSOR(Opc, 22, 24);
    ACCESSOR(Imm12, 10, 22);
    ACCESSOR(Rn, 5, 10);
    ACCESSOR(Rt, 0, 5);

    constexpr InsnLoadStoreRegisterUnsignedImmediate(u8 size, u8 v, u8 opc,
                                                     u16 imm12,
                                                     reg::Register rn,
                                                     reg::Register rt)
        : InsnOpx1x0(Op0) {
        SetOp2(Op2);
        SetSize(size);
        SetV(v);
        SetOpc(opc);
        SetImm12(imm12);
        SetRn(rn.Index());
        SetRt(rt.Index());
    }
};
} // namespace exl::armv8::inst::impl::opx1x0
