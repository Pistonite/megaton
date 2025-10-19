#include "futex.h"
// #include <nn/svc.h>
#include <switch/kernel/svc.h> // libnx
#include <errno.h>
// #include <stdatomic.h> // we don't have stdatomic

// using namespace std;

// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2025 Megaton contributors

// typedef int64_t time_t;

// struct timespec {
//     /// seconds
//     pub tv_sec: time_t,
//     /// nanoseconds
//     pub tv_nsec: i32,
// }

    // extern "C" int futex_wait_impl(unsigned int *address,
    // 	unsigned int expected,
    // 	const timespec* timeout,
    // 	unsigned int flags
    // ) {
    //     return 0;
    // }

    /*
    https://github.com/switchbrew/libnx/blob/master/nx/include/switch/kernel/wait.h
    #include <kernel/wait.h>

    WaitableMethods
    {
        .beginWait = _ueventBeginWait,
        .onTimeout = _ueventOnTimeout,
        .onSignal = _ueventOnSignal,
    };

    Copy implementation of Mutex and Condvar
    What futex does is it will try to lock. It will only involve a mutex 
    */

    /* 
    The  futex()  system  call provides a method for waiting until a certain condition becomes true.  It is typically used as a blocking construct in the context of
    shared-memory synchronization.  When using futexes, the majority of the synchronization operations are performed in user space.  A  user-space  program  employs
    the  futex()  system call only when it is likely that the program has to block for a longer time until the condition becomes true.  Other futex() operations can
    be used to wake any processes or threads waiting for a particular condition.


    Can I use conditional variables for this? They, similarly, provide a method for waiting until a certain condition becomes true.

    What is the condition?
        value at futex word still contains expected value
        The futex_wait operation tests that the value at the futex word pointed to by the address uaddr still contains the expected value val, and if so, then sleeps waiting for
       a FUTEX_WAKE(2const) operation on the futex word.

    What is the futex word?
        On all platforms, futexes are four-byte integers that must be aligned on a four-byte boundary.
       In order to share a futex between processes, the futex is placed in a re‐gion of shared memory, created using (for example) mmap(2) or shmat(2)
       In a multithreaded program, it is sufficient to place the futex word in a global variable shared by all threads.
    
    When executing a futex operation that requests to block a thread, the kernel will block only if the futex word has the value that the  calling  thread  supplied
       (as one of the arguments of the futex() call) as the expected value of the futex word.


    We can use std::atomic
    https://github.com/open-ead/sead/blob/master/include/thread/seadAtomic.h

    Expected value is notification counter
    */

/*
This operation wakes at most val of the waiters that are waiting (e.g., inside FUTEX_WAIT(2const)) on the futex word at the address uaddr.

Most commonly, val is specified as either 1 (wake up a single waiter) or INT_MAX (wake up all waiters).

No  guarantee  is  provided  about  which waiters are awoken (e.g., a waiter with a higher scheduling priority is not guaranteed to be awoken in preference to a
waiter with a lower priority).

pub unsafe extern "C" fn sys_futex_wake(address: *mut u32, count: i32) -> i32
*/


// Result svcWaitForAddress(void *address, u32 arb_type, s64 value, s64 timeout);
// svcWaitForAddress is 0x34
extern "C" int32_t sys_futex_wake(uint32_t *address, int32_t count) {
    if(address == NULL) return -EINVAL; // EINVAL
    int32_t val = (int32_t) *address;
    return svcSignalToAddress((void*) address, SignalType_Signal, val, count);
}

/*
example usage

while true {
    lock mutex
    if compare_and_swap(futex word, 1): 
        break

    let s = sys_futex_wait(futex_word, 1, timeout, flags);
    if(s == -1) error
}

*/


/*
This operation tests that the value at the futex word pointed to
by the address uaddr still contains the expected value val, and if
so, then sleeps waiting for a FUTEX_WAKE(2const) operation on the
futex word.
*/

/*

if address.is_null() {
		return -i32::from(Errno::Inval);
	}

	let address = unsafe { &*(address as *const AtomicU32) };
	let timeout = if timeout.is_null() {
		None
	} else {
		match unsafe { timeout.read().into_usec() } {
			Some(usec) if usec >= 0 => Some(usec as u64),
			_ => return -i32::from(Errno::Inval),
		}
	};
	let Some(flags) = Flags::from_bits(flags) else {
		return -i32::from(Errno::Inval);
	};

	synch::futex_wait(address, expected, timeout, flags)
*/


struct timespec {
    /// seconds
    int64_t tv_sec;
    /// nanoseconds
    int32_t tv_nsec;
};

// returns 1 on succes, 0 on error
int to_usec(timespec* timespec, uint64_t *out) {
    // Source: https://github.com/hermit-os/kernel/blob/0fa55cc454e5bc0fec38e963563114acf4e9265a/src/time.rs#L65
    // self.tv_sec
	// 		.checked_mul(1_000_000)
	// 		.and_then(|usec| usec.checked_add((self.tv_nsec / 1000).into()))

    // TODO: Check for overflow
    *out = (uint64_t) timespec->tv_sec * 1000000; 
    *out += timespec->tv_nsec / 1000;
    return 1; 
}

// how does synch::futex_wait compare to svcWaitForAddress?

// pub unsafe extern "C" fn sys_futex_wait(address: *mut u32, expected: u32, timeout: *const timespec, flags: u32) -> i32
extern "C" int32_t sys_futex_wait(uint32_t *address, uint32_t expected, timespec* timeout, uint32_t flags) {
    if(address == NULL) return -EINVAL;
    
    // atomic_uint32_t* atomic_address = (atomic_uint32_t *) address; // TODO: How to cast to atomic?

    uint64_t timeout_usec;
    if(!to_usec(timeout, &timeout_usec)) {
        return -EINVAL;
    }
    if(!(flags & 0b01)) { // absolute timeout -- convert to relative
        uint64_t current_time_usec = 0;
        timeout_usec -= current_time_usec;
    }

    uint32_t result = svcWaitForAddress(address, ArbitrationType_WaitIfEqual, expected, (int64_t) timeout_usec);
    return result;
}
