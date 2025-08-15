
#pragma once

#include <exl/armv8/insn.hpp>

namespace exl::armv8::inst::impl {

struct InsnOp100x : public Insn {

    ACCESSOR(Op0, 23, 26);

    constexpr InsnOp100x(u8 op0) : Insn(0b1000) { SetOp0(op0); }
};
} // namespace exl::armv8::inst::impl
