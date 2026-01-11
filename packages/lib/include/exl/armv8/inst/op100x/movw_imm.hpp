#pragma once

#include <exl/armv8/inst/op100x.hpp>
#include <exl/armv8/register.hpp>

namespace exl::armv8::inst::impl::op100x {

struct InsnMoveWideImm : public InsnOp100x {

    static constexpr u8 Op0 = 0b101;

    ACCESSOR(Sf, 31);
    ACCESSOR(Opc, 29, 31);
    ACCESSOR(Hw, 21, 23);
    ACCESSOR(Imm16, 5, 21);
    ACCESSOR(Rd, 0, 5);

    constexpr InsnMoveWideImm(reg::Register reg, u8 opc, u8 hw, u16 imm)
        : InsnOp100x(Op0) {
        SetSf(reg.Is64());
        SetOpc(opc);
        SetHw(hw);
        SetImm16(imm);
        SetRd(reg.Index());
    }
};
}; // namespace exl::armv8::inst::impl::op100x
