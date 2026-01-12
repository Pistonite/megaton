// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/opx1x0.hpp>
#include <exl/armv8/register.hpp>
#include <exl/armv8/util/sign_extend.hpp>

namespace exl::armv8::inst::impl::opx1x0 {

struct InsnLoadRegisterLiteral : public InsnOpx1x0 {

    static constexpr u8 Op0 = 0b0001;
    static constexpr u8 Op2 = 0b00;

    ACCESSOR(Opc, 30, 31);
    ACCESSOR(V, 26);
    ACCESSOR(Imm19, 5, 24);
    ACCESSOR(Rt, 0, 5);

    constexpr InsnLoadRegisterLiteral(reg::Register rt, int imm19, u8 v, u8 opc)
        : InsnOpx1x0(Op0) {
        SetOp0(Op0);
        SetOp2(Op2);
        SetOpc(opc);
        SetV(v);
        SetImm19(util::SignExtend<Imm19Mask.Count>(imm19));
        SetRt(rt.Index());
    }
};
} // namespace exl::armv8::inst::impl::opx1x0
