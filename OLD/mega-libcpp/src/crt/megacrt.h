#pragma once

#include <switch/types.h>

#ifdef __cplusplus
extern "C" {
#endif


/** Get LibNX-compatible HOS version value using the SDK */
u32 __megacrt_get_hos_version_for_libnx();

/** Get the current thread handle using the SDK */
Handle __megacrt_get_current_thread();

/** Abort the process with a result code */
void __megacrt_abort(int);

/** Init the services */
void __megacrt_init_services();



#ifdef __cplusplus
}
#endif
