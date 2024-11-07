#pragma once

#include <exl/armv8/inst/op100x/pc_rel_addr.hpp>
#include <exl/armv8/register.hpp>

namespace exl::armv8::inst {

struct Adr : public impl::op100x::InsnPcRelAddr {

    static constexpr u8 Opc = 0b00;
    static constexpr u8 Hw = 0b00;

    constexpr Adr(reg::Register reg, u32 imm)
        : InsnPcRelAddr(reg, imm, Op_ADR) {}
};

static_assert(Adr(reg::X0, 0x1000u).Value() == 0x10008000, "");
static_assert(Adr(reg::X1, 0xfff0u).Value() == 0x1007FF81, "");
static_assert(Adr(reg::X2, 0x69669u).Value() == 0x3034B342, "");
} // namespace exl::armv8::inst
