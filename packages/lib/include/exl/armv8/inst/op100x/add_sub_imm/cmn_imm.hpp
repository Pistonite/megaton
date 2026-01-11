// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include "adds_imm.hpp"

namespace exl::armv8::inst {

/* Alias. */
struct CmnImm : public AddsImm {

    static constexpr reg::Register GetRd(reg::Register reg) {
        if (reg.Is64()) {
            return reg::None64;
        } else {
            return reg::None32;
        }
    }

    constexpr CmnImm(reg::Register reg, u32 imm)
        : AddsImm(GetRd(reg), reg, imm) {}
};

static_assert(CmnImm(reg::X0, 45).Value() == 0xB100B41F, "");
static_assert(CmnImm(reg::W1, 32).Value() == 0x3100803F, "");
static_assert(CmnImm(reg::X2, 0x4000).Value() == 0xB140105F, "");
static_assert(CmnImm(reg::X3, 0x54000).Value() == 0xB141507F, "");
} // namespace exl::armv8::inst
