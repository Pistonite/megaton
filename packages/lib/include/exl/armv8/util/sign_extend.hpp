#pragma once

#include <concepts>
#include <cstdint>

namespace exl::util {

namespace impl {
template <std::size_t Bits> struct SignExtender {
    std::intmax_t m_Extended : Bits;
};
} // namespace impl

template <std::size_t Bits, std::integral T> constexpr T SignExtend(T value) {
    impl::SignExtender<Bits> extender{value};
    return extender.m_Extended & ((1 << Bits) - 1);
}

} // namespace exl::util
