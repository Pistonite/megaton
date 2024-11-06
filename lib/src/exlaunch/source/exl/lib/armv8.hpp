#pragma once
#include <megaton/prelude.h>


#include "armv8/util/common.hpp"
#include "armv8/util/bitset.hpp"
#include "armv8/register.hpp"

namespace exl::armv8 {

    using InstType = u32;

    template<InstType... Args>
    using InstMask = util::Mask<InstType, Args...>;

    using InstBitSet = util::BitSet<InstType>;
}

#include "armv8/instructions.hpp"
