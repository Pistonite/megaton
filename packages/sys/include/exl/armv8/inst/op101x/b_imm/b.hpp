#pragma once

#include <exl/armv8/inst/op101x/b_imm.hpp>

namespace exl::armv8::inst {

struct Branch : public impl::op101x::InsnBranchImm {

    constexpr Branch(u32 relative_address)
        : InsnBranchImm(InsnBranchImm::B, relative_address) {}
};

static_assert(Branch(0x4440).Value() == 0x14001110, "");
static_assert(Branch(0x4200).Value() == 0x14001080, "");
static_assert(Branch(0x6900).Value() == 0x14001A40, "");
static_assert(Branch(0x0008).Value() == 0x14000002, "");
} // namespace exl::armv8::inst
