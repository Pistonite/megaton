// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors
// * * * * *
// This file was taken from the exlaunch project and modified.
// See original license information below
//
// SPDX-License-Identifier: GPL-2.0-or-later
// Copyright (c) shadowninja

#include <megaton/align.h>
#include <megaton/prelude.h>

#include <cstring>

extern "C" {
#include <switch/arm/tls.h>
#include <switch/kernel/svc.h>
#include <switch/result.h>
}

#include <megaton/__priv/proc_handle.h>

namespace megaton::__priv {

static const u32 s_send_handle_msg[4] = {0x00000000, 0x80000000, 0x00000002,
                                         CUR_PROCESS_HANDLE};
static Handle s_handle = INVALID_HANDLE;

/** thread that will receive the proc handle */
static noreturn_ recv_handle_thread_main(void* session_handle_ptr) {
    // Convert the argument to a handle we can use.
    Handle session_handle = (Handle)(uintptr_t)session_handle_ptr;

    // Receive the request from the client thread.
    memset(armGetTls(), 0, 0x10);
    s32 idx = 0;
    if (R_FAILED(svcReplyAndReceive(&idx, &session_handle, 1, INVALID_HANDLE,
                                    UINT64_MAX))) {
        panic_("svcReplyAndReceive failed.");
    }

    // Set the process handle.
    s_handle = ((u32*)armGetTls())[3];

    // Close the session.
    svcCloseHandle(session_handle);

    // Terminate ourselves.
    svcExitThread();

    // This code will never execute.
    while (true)
        ;
}

static void get_via_ipc_trick(void) {
    alignas(PAGE_SIZE) u8 temp_thread_stack[0x1000];

    // Create a new session to transfer our process handle to ourself
    Handle server_handle, client_handle;
    if (R_FAILED(svcCreateSession(&server_handle, &client_handle, 0, 0))) {
        panic_("svcCreateSession failed.");
    }

    // Create a new thread to receive our handle.
    Handle thread_handle;
    if (R_FAILED(svcCreateThread(
            &thread_handle, (void*)&recv_handle_thread_main,
            (void*)(uintptr_t)server_handle,
            temp_thread_stack + sizeof(temp_thread_stack), 0x20, 2))) {
        panic_("svcCreateThread failed.");
    }

    // Start the new thread.
    if (R_FAILED(svcStartThread(thread_handle))) {
        panic_("svcStartThread failed.");
    }

    // Send the message.
    memcpy(armGetTls(), s_send_handle_msg, sizeof(s_send_handle_msg));
    svcSendSyncRequest(client_handle);

    // Close the session handle.
    svcCloseHandle(client_handle);

    // Wait for the thread to be done.
    if (R_FAILED(svcWaitSynchronizationSingle(thread_handle, UINT64_MAX))) {
        panic_("svcWaitSynchronizationSingle failed.");
    }

    // Close the thread handle.
    svcCloseHandle(thread_handle);
}

static Result get_via_mesosphere() {
    u64 handle;
    Result r = svcGetInfo(&handle, 65001 /*InfoType_MesosphereCurrentProcess*/,
                          INVALID_HANDLE, 0);
    if (R_FAILED(r)) {
        return r;
    }
    s_handle = handle;

    return 0;
}

Handle current_process() {
    if (s_handle == INVALID_HANDLE) {
        /* Try to ask mesosphere for our process handle. */
        Result r = get_via_mesosphere();

        /* Fallback to an IPC trick if mesosphere is old/not present. */
        if (R_FAILED(r)) {
            get_via_ipc_trick();
        }
    }
    return s_handle;
}
}; // namespace megaton::__priv
