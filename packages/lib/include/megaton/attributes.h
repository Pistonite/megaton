#pragma once
/** #[inline(never)] */
#define inline_never_ __attribute__((noinline))
/** #[inline(always)] */
#define inline_always_ __attribute__((always_inline)) static inline
/** #[inline(always)] for a member function*/
#define inline_member_ __attribute__((always_inline)) inline

/** -> ! */
#define noreturn_ __attribute__((noreturn)) void

/** #[used] */
#define used_ __attribute__((used))
