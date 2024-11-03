//! Printing system
use std::io::IsTerminal;

static mut VERBOSE: bool = false;
static mut COLOR: bool = true;

/// Enable verbose printing
pub fn verbose_on() {
    unsafe { VERBOSE = true }
}

/// Disable colored printing
pub fn color_off() {
    unsafe { COLOR = false }
}

/// Automatically enable colored printing if stderr is a terminal
pub fn auto_color() {
    if !std::io::stderr().is_terminal() {
        color_off();
    }
}

/// Check if colored printing is enabled
#[inline]
pub fn is_colored() -> bool {
    unsafe { COLOR }
}

#[doc(hidden)]
pub mod __priv {
    pub static RED: &str = "\x1b[1;31m";
    pub static GREEN: &str = "\x1b[1;32m";
    pub static YELLOW: &str = "\x1b[1;33m";
    pub static CYAN: &str = "\x1b[1;36m";
    pub static RESET: &str = "\x1b[0m";

    #[inline]
    pub fn is_verbose() -> bool {
        unsafe { super::VERBOSE }
    }
}


/// Log a status line using info color
#[macro_export]
macro_rules! infoln {
    ($status:expr, $($args:tt)*) => {
        {
            use ::std::io::Write;
            let mut s = ::std::io::stderr().lock();
            let status = { $status };
            if $crate::print::is_colored() {
                let _ = write!(&mut s, "{}{:>12}{} ", $crate::print::__priv::GREEN, status, $crate::print::__priv::RESET);
            } else {
                let _ = write!(&mut s, "{:>12} ", status);
            }
            let _ = writeln!(&mut s, $($args)*);
        }
    };
}

/// Log a status line using error color
#[macro_export]
macro_rules! errorln {
    ($status:expr, $($args:tt)*) => {
        {
            use ::std::io::Write;
            let mut s = ::std::io::stderr().lock();
            let status = { $status };
            if $crate::print::is_colored() {
                let _ = write!(&mut s, "{}{:>12}{} ", $crate::print::__priv::RED, status, $crate::print::__priv::RESET);
            } else {
                let _ = write!(&mut s, "{:>12} ", status);
            }
            let _ = writeln!(&mut s, $($args)*);
        }
    };
}

/// Log a status line using hint color
#[macro_export]
macro_rules! hintln {
    ($status:expr, $($args:tt)*) => {
        {
            use ::std::io::Write;
            let mut s = ::std::io::stderr().lock();
            let status = { $status };
            if $crate::print::is_colored() {
                let _ = write!(&mut s, "{}{:>12}{} ", $crate::print::__priv::YELLOW, status, $crate::print::__priv::RESET);
            } else {
                let _ = write!(&mut s, "{:>12} ", status);
            }
            let _ = writeln!(&mut s, $($args)*);
        }
    };
}

/// Log a line using verbose color, if verbose is enabled
#[macro_export]
macro_rules! verboseln {
    ($($args:tt)*) => {
        {
            if $crate::print::__priv::is_verbose() {
                use ::std::io::Write;
                let mut s = ::std::io::stderr().lock();
                if $crate::print::is_colored() {
                    let _ = write!(&mut s, "{}     VERBOSE{} ", $crate::print::__priv::CYAN, $crate::print::__priv::RESET);
                } else {
                    let _ = write!(&mut s, "     VERBOSE ");
                }
                let _ = writeln!(&mut s, $($args)*);
            }
        }
    };
}
