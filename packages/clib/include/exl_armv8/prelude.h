/** Prelude includes for working with armv8 instructions */
#pragma once

#include <exl/armv8/insn.hpp>
#include <exl/armv8/register.hpp>

#include <exl/armv8/inst/op100x/add_sub_imm/add_imm.hpp>
#include <exl/armv8/inst/op100x/add_sub_imm/adds_imm.hpp>
#include <exl/armv8/inst/op100x/add_sub_imm/cmn_imm.hpp>
#include <exl/armv8/inst/op100x/add_sub_imm/cmp_imm.hpp>
#include <exl/armv8/inst/op100x/add_sub_imm/sub_imm.hpp>
#include <exl/armv8/inst/op100x/add_sub_imm/subs_imm.hpp>
#include <exl/armv8/inst/op100x/movw_imm/movk.hpp>
#include <exl/armv8/inst/op100x/movw_imm/movn.hpp>
#include <exl/armv8/inst/op100x/movw_imm/movz.hpp>
#include <exl/armv8/inst/op100x/pc_rel_addr/adr.hpp>
#include <exl/armv8/inst/op100x/pc_rel_addr/adrp.hpp>

#include <exl/armv8/inst/op101x/b_imm/b.hpp>
#include <exl/armv8/inst/op101x/b_imm/bl.hpp>
#include <exl/armv8/inst/op101x/b_reg/br.hpp>
#include <exl/armv8/inst/op101x/b_reg/ret.hpp>
#include <exl/armv8/inst/op101x/hints/nop.hpp>

#include <exl/armv8/inst/opx101/logical_shifted_register/mov_register.hpp>
#include <exl/armv8/inst/opx101/logical_shifted_register/orr_shifted_register.hpp>

#include <exl/armv8/inst/opx1x0/load_register_literal/ldr_literal.hpp>
#include <exl/armv8/inst/opx1x0/load_store_register_offset/ldr_register_offset.hpp>
#include <exl/armv8/inst/opx1x0/load_store_register_offset/str_register_offset.hpp>
#include <exl/armv8/inst/opx1x0/load_store_register_unscaled_immediate/ldur_unscaled_immediate.hpp>
#include <exl/armv8/inst/opx1x0/load_store_register_unscaled_immediate/stur_unscaled_immediate.hpp>
#include <exl/armv8/inst/opx1x0/load_store_register_unsigned_immediate/ldr_register_immediate.hpp>
#include <exl/armv8/inst/opx1x0/load_store_register_unsigned_immediate/str_register_immediate.hpp>
