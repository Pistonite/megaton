#pragma once

#define APPEND_IMPL(x, y) x ## y
#define APPEND(x, y) APPEND_IMPL(x, y)

#define NUM_ARGS_(_1, _2, _3, _4, _5, _6, TOTAL, ...) TOTAL
#define NUM_ARGS(...) NUM_ARGS_(__VA_ARGS__, 6, 5, 4, 3, 2, 1)
#define VA_MACRO(MACRO, ...) APPEND(MACRO, NUM_ARGS(__VA_ARGS__))(__VA_ARGS__)

// changed
