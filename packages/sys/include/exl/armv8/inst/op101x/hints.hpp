#pragma once

#include <exl/armv8/inst/op101x.hpp>

namespace exl::armv8::inst::impl::op101x {

struct InsnHints : public InsnOp101x {

    static constexpr u8 Op0 = 0b110;
    static constexpr u16 Op1 = 0b01000000110010;
    static constexpr u8 Op2 = 0b11111;

    ACCESSOR(CRm, 8, 12);
    ACCESSOR(LocalOp2, 5, 8);

    constexpr InsnHints() : InsnOp101x(Op0) {
        SetOp1(Op1);
        SetOp2(Op2);
    }
};
} // namespace exl::armv8::inst::impl::op101x
