#pragma once

#include <exl/armv8/inst/op101x/b_reg.hpp>
#include <exl/armv8/register.hpp>

namespace exl::armv8::inst {

struct Ret : public impl::op101x::InsnBranchRegister {

    static constexpr u8 Opc = 0b0010;
    static constexpr u8 Op2 = 0b11111;
    static constexpr u8 Op3 = 0b000000;
    static constexpr u8 Op4 = 0b00000;

    constexpr Ret(reg::Register rn = reg::X30) : InsnBranchRegister(Opc, Op2) {
        /*
            static_assert(rn.Is64());
        */

        SetOp3(Op3);
        SetRn(rn.Index());
        SetOp4(Op4);
    }
};

static_assert(Ret(reg::X0).Value() == 0xD65F0000, "");
static_assert(Ret(reg::X1).Value() == 0xD65F0020, "");
static_assert(Ret(reg::X2).Value() == 0xD65F0040, "");
static_assert(Ret(reg::X30).Value() == 0xD65F03C0, "");
static_assert(Ret().Value() == 0xD65F03C0, "");
} // namespace exl::armv8::inst
