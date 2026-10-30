
/** Binding for hermit kernel implementation in Rust to call into the C side */

#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void _megahermit_abort();
void _megahermit_stub1();
void _megahermit_stub2();


#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
