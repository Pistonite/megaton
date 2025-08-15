//! Printing system
use std::io::{BufReader, IsTerminal, Read, Write};

use unicode_reader::Graphemes;

static mut VERBOSE: bool = false;
static mut COLOR: bool = true;
static mut GIT_COLOR_FLAG: &str = "color.ui=always";

/// Enable verbose printing
pub fn verbose_on() {
    unsafe { VERBOSE = true }
}

/// Disable colored printing
pub fn color_off() {
    unsafe { COLOR = false }
    unsafe { GIT_COLOR_FLAG = "color.ui=never" }
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

/// Get the color switch value for the `--color` flag
/// for various tools
pub fn color_flag() -> &'static str {
    if is_colored() {
        "always"
    } else {
        "never"
    }
}

/// Get the git color switch value for the `-c` flag
pub fn git_color_flag() -> &'static str {
    unsafe { GIT_COLOR_FLAG }
}

#[doc(hidden)]
pub mod __priv {
    pub static RED: &str = "\x1b[1;31m";
    pub static GREEN: &str = "\x1b[1;32m";
    pub static YELLOW: &str = "\x1b[1;33m";
    pub static CYAN: &str = "\x1b[1;36m";
    pub static MAGENTA: &str = "\x1b[1;35m";
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
                    let _ = write!(&mut s, "{}     VERBOSE{} ", $crate::print::__priv::MAGENTA, $crate::print::__priv::RESET);
                } else {
                    let _ = write!(&mut s, "     VERBOSE ");
                }
                let _ = writeln!(&mut s, $($args)*);
            }
        }
    };
}

/// Progress printer
#[derive(Debug)]
pub struct Progress<R: Read> {
    writer: ProgressWriter,
    stream: BufReader<R>,
}

impl<R: Read> Progress<R> {
    pub fn new(tag: &str, stream: BufReader<R>) -> Self {
        let term_width = if std::io::stderr().is_terminal() {
            match terminal_size::terminal_size() {
                Some((w, _)) => w.0 as usize,
                None => 0,
            }
        } else {
            0
        };

        Self {
            writer: ProgressWriter {
                tag: tag.to_string(),
                term_width,
            },
            stream,
        }
    }

    /// Dump the progress to stderr
    ///
    /// This will block until the stream is closed.
    /// Call on a separate thread if needed.
    /// If stderr is not a terminal, only the last line(s) will be printed.
    pub fn dump(self) {
        let mut buf = String::new();

        let iter = Graphemes::from(self.stream);
        for x in iter.map_while(std::result::Result::ok) {
            match x.as_bytes().first() {
                None => {}
                Some(b'\r') => {
                    if !buf.is_empty() {
                        self.writer.write_progress(&buf);
                        buf.clear();
                    }
                }
                Some(b'\n') => {
                    if !buf.is_empty() {
                        self.writer.write_perm(&buf);
                        buf.clear();
                    }
                }
                Some(x) if x.is_ascii() => buf.push(*x as char),
                // only allow ascii to avoid issue with
                // rendering unicode characters,
                // since that could break the progress
                _ => buf.push('?'),
            }
        }
        if !buf.is_empty() {
            self.writer.write_perm(&buf);
        }
    }
}

#[derive(Debug)]
struct ProgressWriter {
    pub tag: String,
    pub term_width: usize,
}

impl ProgressWriter {
    pub fn is_terminal(&self) -> bool {
        // width needs to be greater than TAG + SPACE + MESSAGE + SPACE
        // to have meaningful output
        self.term_width >= (12 + 1 + 4 + 1)
    }
    /// Write a progress line. Line does not have line endings
    pub fn write_progress(&self, line: &str) {
        if !self.is_terminal() {
            return;
        }
        let mut s = ::std::io::stderr().lock();
        // clear current line and restart
        if is_colored() {
            let _ = write!(
                &mut s,
                "\x1b[1K\r{}{:>11}{}> ",
                __priv::CYAN,
                self.tag,
                __priv::RESET
            );
        } else {
            let _ = write!(&mut s, "\x1b[1K\r{:>11}> ", self.tag);
        }

        let max_msg_len = self.term_width - 12 - 2;
        if line.len() > max_msg_len {
            // safety: max_msg_len is at least 4 because of is_terminal check
            let _ = write!(&mut s, "{}...", &line[..max_msg_len - 3]);
        } else {
            let _ = write!(&mut s, "{}", line);
        };
        let _ = s.flush();
    }

    /// Write a permanent line. Line does not have line endings
    pub fn write_perm(&self, line: &str) {
        let mut s = ::std::io::stderr().lock();
        if self.is_terminal() {
            let _ = write!(&mut s, "\x1b[1K\r");
        }
        if is_colored() {
            let _ = write!(
                &mut s,
                "{}{:>12}{} ",
                __priv::GREEN,
                self.tag,
                __priv::RESET
            );
        } else {
            let _ = write!(&mut s, "{:>12} ", self.tag);
        }
        let _ = writeln!(&mut s, "{}", line);
    }
}
