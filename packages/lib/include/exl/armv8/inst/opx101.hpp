#pragma once

#include <exl/armv8/insn.hpp>

namespace exl::armv8::inst::impl {

struct InsnOpx101 : public Insn {

    ACCESSOR(Op0, 30);
    ACCESSOR(Op1, 28);
    ACCESSOR(Op2, 20, 24);
    ACCESSOR(Op3, 10, 15);

    constexpr InsnOpx101(u8 op0, u8 op1, u8 op2, u8 op3) : Insn(0b0101) {
        SetOp0(op0);
        SetOp1(op1);
        SetOp2(op2);
        SetOp3(op3);
    }
};
} // namespace exl::armv8::inst::impl
