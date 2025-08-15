#pragma once

#include <exl/armv8/inst/op100x.hpp>

namespace exl::armv8::inst::impl::op100x {

struct InsnAddSubImm : public InsnOp100x {

    static constexpr u8 Op0 = 0b010;

    static const u32 ImmShift = 12;
    static const u32 MaskForImmShift = (1 << ImmShift) - 1;

    ACCESSOR(Sf, 31);
    ACCESSOR(Op, 30);
    ACCESSOR(S, 29);
    ACCESSOR(Sh, 22);
    ACCESSOR(Imm12, 10, 22);
    ACCESSOR(Rn, 5, 10);
    ACCESSOR(Rd, 0, 5);

    constexpr InsnAddSubImm(bool sf, bool op, bool s) : InsnOp100x(Op0) {
        SetSf(sf);
        SetOp(op);
        SetS(s);
    }

    static constexpr bool CalcSh(u32 imm) {
        return imm != 0 && (imm & MaskForImmShift) == 0;
    }

    static constexpr u16 CalcImm(u32 imm) {
        if (CalcSh(imm)) {
            imm >>= ImmShift;
        }
        return static_cast<u16>(imm);
    }
};
}; // namespace exl::armv8::inst::impl::op100x
