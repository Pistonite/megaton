#pragma once

#include <exl/armv8/insn.hpp>

namespace exl::armv8::inst::impl {

struct InsnOp101x : public Insn {

    ACCESSOR(Op0, 29, 32);
    ACCESSOR(Op1, 12, 26);
    ACCESSOR(Op2, 0, 5);

    constexpr InsnOp101x(u8 op0) : Insn(0b1010) { SetOp0(op0); }
};
} // namespace exl::armv8::inst::impl
