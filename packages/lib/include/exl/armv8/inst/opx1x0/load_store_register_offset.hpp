#pragma once

#include <exl/armv8/inst/opx1x0.hpp>

namespace exl::armv8::inst::impl::opx1x0 {

struct InsnLoadStoreRegisterOffset : public InsnOpx1x0 {

    static constexpr u8 Op0 = 0b0011;
    static constexpr u16 Op1 = 0;
    static constexpr u8 Op2 = 0b00;
    static constexpr u8 Op3 = 0b100000;
    static constexpr u8 Op4 = 0b10;

    ACCESSOR(Size, 30, 32);
    ACCESSOR(V, 26);
    ACCESSOR(Opc, 22, 24);
    ACCESSOR(Rm, 16, 21);
    ACCESSOR(Option, 13, 16);
    ACCESSOR(S, 12);
    ACCESSOR(Rn, 5, 10);
    ACCESSOR(Rt, 0, 5);

    constexpr InsnLoadStoreRegisterOffset(u8 size, u8 v, u8 opc)
        : InsnOpx1x0(Op0) {
        SetSize(size);
        SetOpc(opc);
        SetOp1(Op1);
        SetOp2(Op2);
        SetOp3(Op3);
        SetOp4(Op4);
        SetV(v);
    }
};
} // namespace exl::armv8::inst::impl::opx1x0
