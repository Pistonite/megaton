
// here we implement the hermit syscall C API like sys_abort, sys_alloc, etc
//

use crate::megacpp_sys;

extern "C" fn sys_abort() {
    unsafe {
        megacpp_sys::_megahermit_abort();
    }
}
