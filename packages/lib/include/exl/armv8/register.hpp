// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once
#include <megaton/types.h>

namespace exl::armv8::reg {

enum class RegisterKind : bool { W, X };

class Register {
private:
    /* Pack members efficiently. */
    RegisterKind m_Kind : 1;
    char m_Index : 7;

public:
    constexpr Register(RegisterKind kind, u8 index)
        : m_Kind(kind), m_Index(index) {}

    [[nodiscard]] constexpr bool Is32() const { return m_Kind == RegisterKind::W; }
    [[nodiscard]] constexpr bool Is64() const { return m_Kind == RegisterKind::X; }
    [[nodiscard]] constexpr u8 Index() const { return m_Index; }
};
static_assert(sizeof(Register) == 1, "Register");

#define REG(I)                                                                 \
    constexpr inline Register W##I(RegisterKind::W, I);                        \
    constexpr inline Register X##I(RegisterKind::X, I);

// NOLINTBEGIN
REG(0);
REG(1);
REG(2);
REG(3);
REG(4);
REG(5);
REG(6);
REG(7);
REG(8);
REG(9);
// NOLINTEND
REG(10);
REG(11);
REG(12);
REG(13);
REG(14);
REG(15);
REG(16);
REG(17);
REG(18);
REG(19);
REG(20);
REG(21);
REG(22);
REG(23);
REG(24);
REG(25);
REG(26);
REG(27);
REG(28);
REG(29);
REG(30);

// NOLINTNEXTLINE(readability-identifier-length)
constexpr inline auto LR = X30;
// NOLINTNEXTLINE(readability-identifier-length)
constexpr inline auto SP = Register(RegisterKind::X, 31);
constexpr inline auto None32 = Register(RegisterKind::W, -1);
constexpr inline auto None64 = Register(RegisterKind::X, -1);

#undef REG
} // namespace exl::armv8::reg
