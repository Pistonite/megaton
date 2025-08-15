#pragma once

#include <exl/armv8/insn.hpp>

namespace exl::armv8::inst::impl {

struct InsnOpx1x0 : public Insn {

    ACCESSOR(Op0, 28, 32);
    ACCESSOR(Op1, 26);
    ACCESSOR(Op2, 23, 25);
    ACCESSOR(Op3, 16, 22);
    ACCESSOR(Op4, 10, 12);

    constexpr InsnOpx1x0(u8 op0) : Insn(0b0100) { SetOp0(op0); }
};
} // namespace exl::armv8::inst::impl
