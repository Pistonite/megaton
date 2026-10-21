
/** Binding for hermit kernel implementation in Rust to call into the C side */

#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void _megahermit_abort();


#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
