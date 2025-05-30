/*
Copyright 2017-2018 libnx Authors

Permission to use, copy, modify, and/or distribute this software for 
any purpose with or without fee is hereby granted, provided that 
the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES 
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF 
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR 
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES 
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN 
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF 
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/
.macro SVC_BEGIN name
	.section .text.\name, "ax", %progbits
	.global \name
	.type \name, %function
	.align 2
	.cfi_startproc
\name:
.endm

.macro SVC_END
	.cfi_endproc
.endm

SVC_BEGIN svcSetHeapSize
	str x0, [sp, #-16]!
	svc 0x1
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcSetMemoryPermission
	svc 0x2
	ret
SVC_END

SVC_BEGIN svcSetMemoryAttribute
	svc 0x3
	ret
SVC_END

SVC_BEGIN svcMapMemory
	svc 0x4
	ret
SVC_END

SVC_BEGIN svcUnmapMemory
	svc 0x5
	ret
SVC_END

SVC_BEGIN svcQueryMemory
	str x1, [sp, #-16]!
	svc 0x6
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcExitProcess
	svc 0x7
	ret
SVC_END

SVC_BEGIN svcCreateThread
	str x0, [sp, #-16]!
	svc 0x8
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcStartThread
	svc 0x9
	ret
SVC_END

SVC_BEGIN svcExitThread
	svc 0xA
	ret
SVC_END

SVC_BEGIN svcSleepThread
	svc 0xB
	ret
SVC_END

SVC_BEGIN svcGetThreadPriority
	str x0, [sp, #-16]!
	svc 0xC
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcSetThreadPriority
	svc 0xD
	ret
SVC_END

SVC_BEGIN svcGetThreadCoreMask
	stp x0, x1, [sp, #-16]!
	svc 0xE
	ldp x3, x4, [sp], #16
	str w1, [x3]
	str x2, [x4]
	ret
SVC_END

SVC_BEGIN svcSetThreadCoreMask
	svc 0xF
	ret
SVC_END

SVC_BEGIN svcGetCurrentProcessorNumber
	svc 0x10
	ret
SVC_END

SVC_BEGIN svcSignalEvent
	svc 0x11
	ret
SVC_END

SVC_BEGIN svcClearEvent
	svc 0x12
	ret
SVC_END

SVC_BEGIN svcMapSharedMemory
	svc 0x13
	ret
SVC_END

SVC_BEGIN svcUnmapSharedMemory
	svc 0x14
	ret
SVC_END

SVC_BEGIN svcCreateTransferMemory
	str x0, [sp, #-16]!
	svc 0x15
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcCloseHandle
	svc 0x16
	ret
SVC_END

SVC_BEGIN svcResetSignal
	svc 0x17
	ret
SVC_END

SVC_BEGIN svcWaitSynchronization
	str x0, [sp, #-16]!
	svc 0x18
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcCancelSynchronization
	svc 0x19
	ret
SVC_END

SVC_BEGIN svcArbitrateLock
	svc 0x1A
	ret
SVC_END

SVC_BEGIN svcArbitrateUnlock
	svc 0x1B
	ret
SVC_END

SVC_BEGIN svcWaitProcessWideKeyAtomic
	svc 0x1C
	ret
SVC_END

SVC_BEGIN svcSignalProcessWideKey
	svc 0x1D
	ret
SVC_END

SVC_BEGIN svcGetSystemTick
	svc 0x1E
	ret
SVC_END

SVC_BEGIN svcConnectToNamedPort
	str x0, [sp, #-16]!
	svc 0x1F
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcSendSyncRequestLight
	svc 0x20
	ret
SVC_END

SVC_BEGIN svcSendSyncRequest
	svc 0x21
	ret
SVC_END

SVC_BEGIN svcSendSyncRequestWithUserBuffer
	svc 0x22
	ret
SVC_END

SVC_BEGIN svcSendAsyncRequestWithUserBuffer
	str x0, [sp, #-16]!
	svc 0x23
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcGetProcessId
	str x0, [sp, #-16]!
	svc 0x24
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcGetThreadId
	str x0, [sp, #-16]!
	svc 0x25
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcBreak
	svc 0x26
	ret
SVC_END

SVC_BEGIN svcOutputDebugString
	svc 0x27
	ret
SVC_END

SVC_BEGIN svcReturnFromException
	svc 0x28
	ret
SVC_END

SVC_BEGIN svcGetInfo
	str x0, [sp, #-16]!
	svc 0x29
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcFlushEntireDataCache
	svc 0x2A
	ret
SVC_END

SVC_BEGIN svcFlushDataCache
	svc 0x2B
	ret
SVC_END

SVC_BEGIN svcMapPhysicalMemory
	svc 0x2C
	ret
SVC_END

SVC_BEGIN svcUnmapPhysicalMemory
	svc 0x2D
	ret
SVC_END

SVC_BEGIN svcGetDebugFutureThreadInfo
	stp x0, x1, [sp, #-16]!
	svc 0x2E
	ldp x6, x7, [sp], #16
	stp x1, x2, [x6]
	stp x3, x4, [x6, #16]
	str x5, [x7]
	ret
SVC_END

SVC_BEGIN svcGetLastThreadInfo
	stp x1, x2, [sp, #-16]!
	str x0, [sp, #-16]!
	svc 0x2F
	ldr x7, [sp], #16
	stp x1, x2, [x7]
	stp x3, x4, [x7, #16]
	ldp x1, x2, [sp], #16
	str x5, [x1]
	str w6, [x2]
	ret
SVC_END

SVC_BEGIN svcGetResourceLimitLimitValue
	str x0, [sp, #-16]!
	svc 0x30
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcGetResourceLimitCurrentValue
	str x0, [sp, #-16]!
	svc 0x31
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcSetThreadActivity
	svc 0x32
	ret
SVC_END

SVC_BEGIN svcGetThreadContext3
	svc 0x33
	ret
SVC_END

SVC_BEGIN svcWaitForAddress
	svc 0x34
	ret
SVC_END

SVC_BEGIN svcSignalToAddress
	svc 0x35
	ret
SVC_END

SVC_BEGIN svcSynchronizePreemptionState
	svc 0x36
	ret
SVC_END

SVC_BEGIN svcGetResourceLimitPeakValue
	str x0, [sp, #-16]!
	svc 0x37
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcCreateIoPool
	str x0, [sp, #-16]!
	svc 0x39
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcCreateIoRegion
	str x0, [sp, #-16]!
	svc 0x3A
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcDumpInfo
	svc 0x3C
	ret
SVC_END

SVC_BEGIN svcKernelDebug
	svc 0x3C
	ret
SVC_END

SVC_BEGIN svcChangeKernelTraceState
	svc 0x3D
	ret
SVC_END

SVC_BEGIN svcCreateSession
	stp x0, x1, [sp, #-16]!
	svc 0x40
	ldp x3, x4, [sp], #16
	str w1, [x3]
	str w2, [x4]
	ret
SVC_END

SVC_BEGIN svcAcceptSession
	str x0, [sp, #-16]!
	svc 0x41
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcReplyAndReceiveLight
	svc 0x42
	ret
SVC_END

SVC_BEGIN svcReplyAndReceive
	str x0, [sp, #-16]!
	svc 0x43
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcReplyAndReceiveWithUserBuffer
	str x0, [sp, #-16]!
	svc 0x44
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcCreateEvent
	stp x0, x1, [sp, #-16]!
	svc 0x45
	ldp x3, x4, [sp], #16
	str w1, [x3]
	str w2, [x4]
	ret
SVC_END

SVC_BEGIN svcMapIoRegion
	svc 0x46
	ret
SVC_END

SVC_BEGIN svcUnmapIoRegion
	svc 0x47
	ret
SVC_END

SVC_BEGIN svcMapPhysicalMemoryUnsafe
	svc 0x48
	ret
SVC_END

SVC_BEGIN svcUnmapPhysicalMemoryUnsafe
	svc 0x49
	ret
SVC_END

SVC_BEGIN svcSetUnsafeLimit
	svc 0x4A
	ret
SVC_END

SVC_BEGIN svcCreateCodeMemory
	str x0, [sp, #-16]!
	svc 0x4B
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcControlCodeMemory
	svc 0x4C
	ret
SVC_END

SVC_BEGIN svcSleepSystem
	svc 0x4D
	ret
SVC_END

SVC_BEGIN svcReadWriteRegister
	str x0, [sp, #-16]!
	svc 0x4E
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcSetProcessActivity
	svc 0x4F
	ret
SVC_END

SVC_BEGIN svcCreateSharedMemory
	str x0, [sp, #-16]!
	svc 0x50
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcMapTransferMemory
	svc 0x51
	ret
SVC_END

SVC_BEGIN svcUnmapTransferMemory
	svc 0x52
	ret
SVC_END

SVC_BEGIN svcCreateInterruptEvent
	str x0, [sp, #-16]!
	svc 0x53
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcQueryPhysicalAddress
	str x0, [sp, #-16]!
	svc 0x54
	ldr x4, [sp], #16
	stp x1, x2, [x4]
	str x3, [x4, #16]
	ret
SVC_END

SVC_BEGIN svcQueryMemoryMapping
	stp x0, x1, [sp, #-16]!
	svc 0x55
	ldp x3, x4, [sp], #16
	str x1, [x3]
	str x2, [x4]
	ret
SVC_END

SVC_BEGIN svcLegacyQueryIoMapping
	str x0, [sp, #-16]!
	svc 0x55
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcCreateDeviceAddressSpace
	str x0, [sp, #-16]!
	svc 0x56
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcAttachDeviceAddressSpace
	svc 0x57
	ret
SVC_END

SVC_BEGIN svcDetachDeviceAddressSpace
	svc 0x58
	ret
SVC_END

SVC_BEGIN svcMapDeviceAddressSpaceByForce
	svc 0x59
	ret
SVC_END

SVC_BEGIN svcMapDeviceAddressSpaceAligned
	svc 0x5A
	ret
SVC_END

SVC_BEGIN svcMapDeviceAddressSpace
	str x0, [sp, #-16]!
	svc 0x5B
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcUnmapDeviceAddressSpace
	svc 0x5C
	ret
SVC_END

SVC_BEGIN svcInvalidateProcessDataCache
	svc 0x5D
	ret
SVC_END

SVC_BEGIN svcStoreProcessDataCache
	svc 0x5E
	ret
SVC_END

SVC_BEGIN svcFlushProcessDataCache
	svc 0x5F
	ret
SVC_END

SVC_BEGIN svcDebugActiveProcess
	str x0, [sp, #-16]!
	svc 0x60
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcBreakDebugProcess
	svc 0x61
	ret
SVC_END

SVC_BEGIN svcTerminateDebugProcess
	svc 0x62
	ret
SVC_END

SVC_BEGIN svcGetDebugEvent
	svc 0x63
	ret
SVC_END

SVC_BEGIN svcLegacyContinueDebugEvent
	svc 0x64
	ret
SVC_END

SVC_BEGIN svcContinueDebugEvent
	svc 0x64
	ret
SVC_END

SVC_BEGIN svcGetProcessList
	str x0, [sp, #-16]!
	svc 0x65
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcGetThreadList
	str x0, [sp, #-16]!
	svc 0x66
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcGetDebugThreadContext
	svc 0x67
	ret
SVC_END

SVC_BEGIN svcSetDebugThreadContext
	svc 0x68
	ret
SVC_END

SVC_BEGIN svcQueryDebugProcessMemory
	str x1, [sp, #-16]!
	svc 0x69
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcReadDebugProcessMemory
	svc 0x6A
	ret
SVC_END

SVC_BEGIN svcWriteDebugProcessMemory
	svc 0x6B
	ret
SVC_END

SVC_BEGIN svcSetHardwareBreakPoint
	svc 0x6C
	ret
SVC_END

SVC_BEGIN svcGetDebugThreadParam
	stp x0, x1, [sp, #-16]!
	svc 0x6D
	ldp x3, x4, [sp], #16
	str x1, [x3]
	str w2, [x4]
	ret
SVC_END

SVC_BEGIN svcGetSystemInfo
	str x0, [sp, #-16]!
	svc 0x6F
	ldr x2, [sp], #16
	str x1, [x2]
	ret
SVC_END

SVC_BEGIN svcCreatePort
	stp x0, x1, [sp, #-16]!
	svc 0x70
	ldp x3, x4, [sp], #16
	str w1, [x3]
	str w2, [x4]
	ret
SVC_END

SVC_BEGIN svcManageNamedPort
	str x0, [sp, #-16]!
	svc 0x71
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcConnectToPort
	str x0, [sp, #-16]!
	svc 0x72
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcSetProcessMemoryPermission
	svc 0x73
	ret
SVC_END

SVC_BEGIN svcMapProcessMemory
	svc 0x74
	ret
SVC_END

SVC_BEGIN svcUnmapProcessMemory
	svc 0x75
	ret
SVC_END

SVC_BEGIN svcQueryProcessMemory
	str x1, [sp, #-16]!
	svc 0x76
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcMapProcessCodeMemory
	svc 0x77
	ret
SVC_END

SVC_BEGIN svcUnmapProcessCodeMemory
	svc 0x78
	ret
SVC_END

SVC_BEGIN svcCreateProcess
	str x0, [sp, #-16]!
	svc 0x79
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcStartProcess
	svc 0x7A
	ret
SVC_END

SVC_BEGIN svcTerminateProcess
	svc 0x7B
	ret
SVC_END

SVC_BEGIN svcGetProcessInfo
	str x0, [sp, #-16]!
	svc 0x7C
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcCreateResourceLimit
	str x0, [sp, #-16]!
	svc 0x7D
	ldr x2, [sp], #16
	str w1, [x2]
	ret
SVC_END

SVC_BEGIN svcSetResourceLimitLimitValue
	svc 0x7E
	ret
SVC_END

SVC_BEGIN svcCallSecureMonitor
	str x0, [sp, #-16]!
	mov x8, x0
	ldp x0, x1, [x8]
	ldp x2, x3, [x8, #0x10]
	ldp x4, x5, [x8, #0x20]
	ldp x6, x7, [x8, #0x30]
	svc 0x7F
	ldr x8, [sp], #16
	stp x0, x1, [x8]
	stp x2, x3, [x8, #0x10]
	stp x4, x5, [x8, #0x20]
	stp x6, x7, [x8, #0x30]
	ret
SVC_END

SVC_BEGIN svcMapInsecurePhysicalMemory
	svc 0x90
	ret
SVC_END

SVC_BEGIN svcUnmapInsecurePhysicalMemory
	svc 0x91
	ret
SVC_END
