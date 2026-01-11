
#pragma once

#define APPEND_IMPL(x, y) x##y
#define APPEND(x, y) APPEND_IMPL(x, y)

#define NUM_ARGS_(_1, _2, _3, _4, _5, _6, TOTAL, ...) TOTAL
#define NUM_ARGS(...) NUM_ARGS_(__VA_ARGS__, 6, 5, 4, 3, 2, 1)
#define VA_MACRO(MACRO, ...) APPEND(MACRO, NUM_ARGS(__VA_ARGS__))(__VA_ARGS__)

#define _ACCESSOR_MASK_NAME(name) APPEND(name, Mask)
#define _ACCESSOR_GETTER_NAME(name) Get##name
#define _ACCESSOR_SETTER_NAME(name) Set##name

#define _ACCESSOR_BODY(name)                                                   \
    constexpr InstType _ACCESSOR_GETTER_NAME(name)() const {                   \
        return BitsOf<_ACCESSOR_MASK_NAME(name)>();                            \
    }                                                                          \
    constexpr void _ACCESSOR_SETTER_NAME(name)(InstType val) {                 \
        SetBits<_ACCESSOR_MASK_NAME(name)>(val);                               \
    }

#define _ACCESSOR_2(name, low)                                                 \
    static constexpr auto _ACCESSOR_MASK_NAME(name) = InstMask<low>();         \
    _ACCESSOR_BODY(name)
#define _ACCESSOR_3(name, low, high)                                           \
    static constexpr auto _ACCESSOR_MASK_NAME(name) = InstMask<low, high>();   \
    _ACCESSOR_BODY(name)

#define ACCESSOR(...) VA_MACRO(_ACCESSOR_, __VA_ARGS__)
