
/** Binding for hermit kernel implementation in Rust to call into the C side */

#pragma once

// FIXME: clang-tidy is confused about the language of this file,
// add cmake rules for headers
// NOLINTBEGIN(modernize-deprecated-headers)
#include <stdint.h>
// NOLINTEND(modernize-deprecated-headers)

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void _megahermit_abort();
void _megahermit_stub1();
void _megahermit_stub2();

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
