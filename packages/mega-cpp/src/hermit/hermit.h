
/** Binding for hermit kernel implementation in Rust to call into the C side */

#pragma once

#ifdef __cplusplus
#include <cstdint>
#else
#include <stdint.h>
#endif
// FIXME: clang-tidy is confused about the language of this file,
// add cmake rules for headers
// NOLINTBEGIN(modernize-deprecated-headers)
// NOLINTEND(modernize-deprecated-headers)

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void _megahermit_abort();
uint32_t _megahermit_stub1();
void _megahermit_stub2();

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
