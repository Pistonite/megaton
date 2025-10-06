#pragma once

#include "subs_imm.hpp"

namespace exl::armv8::inst {

/* Alias. */
struct CmpImm : public SubsImm {

    static constexpr reg::Register GetRd(reg::Register reg) {
        if (reg.Is64()) {
            return reg::None64;
        } else {
            return reg::None32;
        }
    }

    constexpr CmpImm(reg::Register reg, u32 imm)
        : SubsImm(GetRd(reg), reg, imm) {}
};

static_assert(CmpImm(reg::X0, 45).Value() == 0xF100B41F, "");
static_assert(CmpImm(reg::W1, 32).Value() == 0x7100803F, "");
static_assert(CmpImm(reg::X2, 0x4000).Value() == 0xF140105F, "");
static_assert(CmpImm(reg::X3, 0x54000).Value() == 0xF141507F, "");
} // namespace exl::armv8::inst
