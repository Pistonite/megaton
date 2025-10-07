#pragma once

#include <exl/armv8/inst/opx1x0.hpp>
#include <exl/armv8/register.hpp>
#include <exl/armv8/util/sign_extend.hpp>

namespace exl::armv8::inst::impl::opx1x0 {

struct InsnLoadStoreRegisterUnscaledImmediate : public InsnOpx1x0 {

    static constexpr u8 Op0 = 0b0011;
    static constexpr u8 Op2 = 0b00;
    static constexpr u8 Op3 = 0b000000;
    static constexpr u8 Op4 = 0b00;

    ACCESSOR(Size, 30, 32);
    ACCESSOR(V, 26);
    ACCESSOR(Opc, 22, 24);
    ACCESSOR(Imm9, 12, 21);
    ACCESSOR(Rn, 5, 10);
    ACCESSOR(Rt, 0, 5);

    constexpr InsnLoadStoreRegisterUnscaledImmediate(u8 size, u8 v, u8 opc,
                                                     s16 imm9, reg::Register rn,
                                                     reg::Register rt)
        : InsnOpx1x0(Op0) {
        SetOp2(Op2);
        SetOp3(Op3);
        SetOp4(Op4);
        SetSize(size);
        SetV(v);
        SetOpc(opc);
        SetImm9(util::SignExtend<Imm9Mask.Count>(imm9));
        SetRn(rn.Index());
        SetRt(rt.Index());
    }
};
} // namespace exl::armv8::inst::impl::opx1x0
