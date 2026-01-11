// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/op100x/movw_imm.hpp>
#include <exl/armv8/register.hpp>

namespace exl::armv8::inst {

struct Movn : public impl::op100x::InsnMoveWideImm {

    static constexpr u8 Opc = 0b00;
    static constexpr u8 Hw = 0b00;

    constexpr Movn(reg::Register reg, u16 imm)
        : InsnMoveWideImm(reg, Opc, Hw, imm) {}
};
} // namespace exl::armv8::inst
