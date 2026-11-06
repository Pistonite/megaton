// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/op100x/pc_rel_addr.hpp>
#include <exl/armv8/register.hpp>

namespace exl::armv8::inst {

struct Adrp : public impl::op100x::InsnPcRelAddr {

    static constexpr u8 Opc = 0b00;
    static constexpr u8 Hw = 0b00;

    constexpr Adrp(reg::Register reg, u32 imm)
        : InsnPcRelAddr(reg, imm >> 12, Op_ADRP) {}
};

static_assert(Adrp(reg::X0, 0x1000).Value() == 0xB0000000, "");
static_assert(Adrp(reg::X1, 0xfff000).Value() == 0xF0007FE1, "");
static_assert(Adrp(reg::X2, 0x6969000).Value() == 0xB0034B42, "");
} // namespace exl::armv8::inst
