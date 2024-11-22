/**
 * prelude.h should be included by all source files.
 *
 * This includes common primitive types, macros, and attributes
 */
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/**
 * @file switch/types.h
 * @brief Various system types.
 * @copyright libnx Authors
 */
typedef uint8_t u8;       ///<   8-bit unsigned integer.
typedef uint16_t u16;     ///<  16-bit unsigned integer.
typedef uint32_t u32;     ///<  32-bit unsigned integer.
typedef uint64_t u64;     ///<  64-bit unsigned integer.
typedef __uint128_t u128; ///< 128-bit unsigned integer.

typedef int8_t s8;       ///<   8-bit signed integer.
typedef int16_t s16;     ///<  16-bit signed integer.
typedef int32_t s32;     ///<  32-bit signed integer.
typedef int64_t s64;     ///<  64-bit signed integer.
typedef __int128_t s128; ///< 128-bit unsigned integer.

// rust-ish types
typedef s8 i8;
typedef s16 i16;
typedef s32 i32;
typedef s64 i64;
typedef float f32;
typedef double f64;

typedef volatile u8 vu8;     ///<   8-bit volatile unsigned integer.
typedef volatile u16 vu16;   ///<  16-bit volatile unsigned integer.
typedef volatile u32 vu32;   ///<  32-bit volatile unsigned integer.
typedef volatile u64 vu64;   ///<  64-bit volatile unsigned integer.
typedef volatile u128 vu128; ///< 128-bit volatile unsigned integer.

typedef volatile s8 vs8;     ///<   8-bit volatile signed integer.
typedef volatile s16 vs16;   ///<  16-bit volatile signed integer.
typedef volatile s32 vs32;   ///<  32-bit volatile signed integer.
typedef volatile s64 vs64;   ///<  64-bit volatile signed integer.
typedef volatile s128 vs128; ///< 128-bit volatile signed integer.

typedef size_t usize;

/** #[inline(never)] */
#define inline_never_ __attribute__((noinline))
/** #[inline(always)] */
#define inline_always_ __attribute__((always_inline)) static inline

/** -> ! */
#define noreturn_ __attribute__((noreturn)) void

/** #[used] */
#define used_ __attribute__((used))

// prelude headers

#include <megaton/panic_abort.h>
