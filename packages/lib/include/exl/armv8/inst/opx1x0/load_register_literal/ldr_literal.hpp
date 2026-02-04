// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <exl/armv8/inst/opx1x0/load_register_literal.hpp>
#include <exl/armv8/register.hpp>

namespace exl::armv8::inst {

struct LdrLiteral : public impl::opx1x0::InsnLoadRegisterLiteral {

    static constexpr u8 V = 0b0;

    static constexpr u8 GetOpc(reg::Register rt) { return 0b00 | rt.Is64(); }

    constexpr LdrLiteral(reg::Register rt, u32 relative_distance)
        : InsnLoadRegisterLiteral(rt, relative_distance / 4, V, GetOpc(rt)) {}
};

static_assert(LdrLiteral(reg::X0, 0x08).Value() == 0x58000040, "");
static_assert(LdrLiteral(reg::W1, 0x10).Value() == 0x18000081, "");
static_assert(LdrLiteral(reg::X2, 0x18).Value() == 0x580000C2, "");
static_assert(LdrLiteral(reg::W3, 0x20).Value() == 0x18000103, "");
static_assert(LdrLiteral(reg::X4, 0x28).Value() == 0x58000144, "");
static_assert(LdrLiteral(reg::W5, 0x30).Value() == 0x18000185, "");
static_assert(LdrLiteral(reg::X6, 0x38).Value() == 0x580001C6, "");
static_assert(LdrLiteral(reg::W7, 0x40).Value() == 0x18000207, "");
} // namespace exl::armv8::inst
