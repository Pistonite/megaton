// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2026 Megaton contributors

#pragma once

#include <concepts>

// NOLINTBEGIN(bugprone-reserved-identifier)

namespace exl::util {

template <std::integral Underlying, Underlying _Low,
          Underlying _High = _Low + 1>
struct Mask {
    static constexpr Underlying Low = _Low;
    static constexpr Underlying High = _High;
    static constexpr Underlying Count = _High - _Low;
    static constexpr Underlying Value() {
        auto base = (1 << Count) - 1;
        return base << Low;
    }
};

template <std::integral Underlying> class BitSet {
private:
    Underlying m_Data;

public:
    constexpr BitSet() : m_Data() {}
    constexpr BitSet(Underlying data) : m_Data(data) {}

    template <Mask Mask> constexpr Underlying BitsOf() const {
        /* Take out the bits we want. */
        auto value = m_Data & Mask.Value();
        /* Shift down the bits. */
        return value >> Mask.Low;
    }

    template <Mask Mask> constexpr void SetBits(Underlying value) {
        /* Carve out the bits not in the mask. */
        m_Data &= ~Mask.Value();

        /* Prepare value to be written. */
        auto value_shifted = value << Mask.Low;
        value_shifted &= Mask.Value();

        /* OR in the bits. */
        m_Data |= value_shifted;
    }

    /* Wrappers to construct masks. */
    template <Underlying Low, Underlying High>
    constexpr Underlying BitsOf() const {
        return BitsOf<Mask<Underlying, Low, High>>();
    }
    template <Underlying Low, Underlying High>
    constexpr void SetBits(Underlying value) {
        SetBits<Mask<Underlying, Low, High>>(value);
    }

    /* Conversion operators. */
    constexpr Underlying& operator=(const Underlying& value) {
        m_Data = value;
        return this;
    }

    constexpr Underlying Value() const { return m_Data; }
};
}; // namespace exl::util
// NOLINTEND(bugprone-reserved-identifier)
