#pragma once

#include <exl/armv8/inst/op101x/hints.hpp>

namespace exl::armv8::inst {

struct Nop : public impl::op101x::InsnHints {

    static constexpr u8 CRm = 0b0000;
    static constexpr u8 LocalOp2 = 0b000;

    constexpr Nop() : InsnHints() {
        SetCRm(CRm);
        SetLocalOp2(LocalOp2);
    }
};

static_assert(Nop().Value() == 0xD503201F, "");
} // namespace exl::armv8::inst
