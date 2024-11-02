//! Printing system

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

    #[inline]
    pub fn is_colored() -> bool {
        unsafe { super::COLOR }
    }
}


/// Log a status line using info color
#[macro_export]
macro_rules! infoln {
    ($status:expr, $($args:tt)*) => {
        {
            use ::std::io::Write;
            let mut stdout = ::std::io::stdout().lock();
            let status = { $status };
            if $crate::print::__priv::is_colored() {
                let _ = write!(&mut stdout, "{}{:>12}{} ", $crate::print::__priv::GREEN, status, $crate::print::__priv::RESET);
            } else {
                let _ = write!(&mut stdout, "{:>12} ", status);
            }
            let _ = writeln!(&mut stdout, $($args)*);
        }
    };
}

/// Log a status line using error color
#[macro_export]
macro_rules! errorln {
    ($status:expr, $($args:tt)*) => {
        {
            use ::std::io::Write;
            let mut stdout = ::std::io::stdout().lock();
            let status = { $status };
            if $crate::print::__priv::is_colored() {
                let _ = write!(&mut stdout, "{}{:>12}{} ", $crate::print::__priv::RED, status, $crate::print::__priv::RESET);
            } else {
                let _ = write!(&mut stdout, "{:>12} ", status);
            }
            let _ = writeln!(&mut stdout, $($args)*);
        }
    };
}

/// Log a status line using hint color
#[macro_export]
macro_rules! hintln {
    ($status:expr, $($args:tt)*) => {
        {
            use ::std::io::Write;
            let mut stdout = ::std::io::stdout().lock();
            let status = { $status };
            if $crate::print::__priv::is_colored() {
                let _ = write!(&mut stdout, "{}{:>12}{} ", $crate::print::__priv::YELLOW, status, $crate::print::__priv::RESET);
            } else {
                let _ = write!(&mut stdout, "{:>12} ", status);
            }
            let _ = writeln!(&mut stdout, $($args)*);
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
                let mut stdout = ::std::io::stdout().lock();
                if $crate::print::__priv::is_colored() {
                    let _ = write!(&mut stdout, "{}     VERBOSE{} ", $crate::print::__priv::CYAN, $crate::print::__priv::RESET);
                } else {
                    let _ = write!(&mut stdout, "     VERBOSE ");
                }
                let _ = writeln!(&mut stdout, $($args)*);
            }
        }
    };
}
