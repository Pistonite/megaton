use crate::prelude::*;

use std::ffi::{OsStr, OsString};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{ChildStderr, ChildStdin, ChildStdout, ExitStatus, Stdio};

use super::Error;

/// Convenience macro for building an argument list
#[macro_export]
macro_rules! args {
    ($($arg:expr),* $(,)?) => {
        {
            let args: Vec<&std::ffi::OsStr> = vec![$($arg.as_ref()),*];
            args
        }
    };
}

/// Convenience wrapper around `Command` for
/// building a child process
pub struct Command {
    executable: OsString,
    command: std::process::Command,
}

impl Command {
    pub fn new(executable: impl AsRef<OsStr>) -> Self {
        Self {
            executable: executable.as_ref().to_os_string(),
            command: std::process::Command::new(executable),
        }
    }

    #[inline]
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.command.current_dir(dir);
        self
    }

    /// Set args as in `Command`
    #[inline]
    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Self {
        self.command.args(args);
        self
    }

    /// Set stdin to pipe
    #[inline]
    pub fn pipe_stdin(mut self) -> Self {
        self.command.stdin(Stdio::piped());
        self
    }

    /// Set stdout to pipe
    #[inline]
    pub fn pipe_stdout(mut self) -> Self {
        self.command.stdout(Stdio::piped());
        self
    }

    /// Set stderr to pipe
    #[inline]
    pub fn pipe_stderr(mut self) -> Self {
        self.command.stderr(Stdio::piped());
        self
    }

    /// Set stdout and stderr to pipe
    #[inline]
    pub fn piped(self) -> Self {
        self.pipe_stdout().pipe_stderr()
    }

    /// Set stdout to null
    #[inline]
    pub fn silence_stdout(mut self) -> Self {
        self.command.stdout(Stdio::null());
        self
    }

    /// Set stderr to null
    #[inline]
    pub fn silence_stderr(mut self) -> Self {
        self.command.stderr(Stdio::null());
        self
    }

    /// Set stdout and stderr to null
    #[inline]
    pub fn silent(self) -> Self {
        self.silence_stdout().silence_stderr()
    }

    pub fn spawn(mut self) -> Result<Spawned, Error> {
        verboseln!("running {}", self.get_command_string());
        // we don't care about escaping it properly, just for debugging
        let child = self
            .command
            .spawn()
            .change_context_lazy(|| Error::Spawn(self.executable.to_string_lossy().to_string()))?;

        Ok(Spawned {
            command: self,
            child,
        })
    }

    /// Get a string representation of the command for debugging purposes
    pub fn get_command_string(&self) -> String {
        get_command_string(&self.executable, &self.command)
    }
}

/// Handle for a spawned child command
pub struct Spawned {
    command: Command,
    child: std::process::Child,
}

impl Spawned {
    /// Get a string representation of the command for debugging purposes
    pub fn get_command_string(&self) -> String {
        self.command.get_command_string()
    }
    pub fn take_stdin(&mut self) -> ChildStdin {
        self.child
            .stdin
            .take()
            .expect("stdin is not piped! Need to call `pipe_stdin` on the builder!")
    }
    /// Take the stdout of the child process and wrap it in a `BufReader`
    pub fn take_stdout(&mut self) -> Option<BufReader<ChildStdout>> {
        self.child.stdout.take().map(BufReader::new)
    }

    /// Take the stderr of the child process and wrap it in a `BufReader`
    pub fn take_stderr(&mut self) -> Option<BufReader<ChildStderr>> {
        self.child.stderr.take().map(BufReader::new)
    }

    /// Wait for the child process to exit
    pub fn wait(mut self) -> Result<Finished, Error> {
        let status = self
            .child
            .wait()
            .change_context_lazy(|| {
                Error::Subcommand(self.command.executable.to_string_lossy().to_string())
            })
            .attach_printable_lazy(|| format!("running {}", self.get_command_string()))?;

        Ok(Finished {
            status,
            child: self,
        })
    }
}

/// Get a string representation of a command for debugging purposes
fn get_command_string(executable: &OsStr, command: &std::process::Command) -> String {
    let args_str = command
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    format!("{} {}", executable.to_string_lossy(), args_str)
}

/// A finished process
pub struct Finished {
    pub status: ExitStatus,
    child: Spawned,
}

impl Finished {
    /// Take the stderr, and dump it using `errorln!` with the prefix
    ///
    /// Note this does not check the exit status of the child
    pub fn dump_stderr(&mut self, prefix: &str) {
        if let Some(stderr) = self.child.take_stderr() {
            for line in stderr.lines().map_while(std::result::Result::ok) {
                errorln!(prefix, "{}", line);
            }
        }
    }

    /// Check if the process was successful
    pub fn is_success(&self) -> bool {
        self.status.success()
    }

    /// Check the exit status of the process, return an error if it failed
    pub fn check(&self) -> Result<(), Error> {
        if self.is_success() {
            Ok(())
        } else {
            Err(Error::ExitStatus(
                self.child.command.executable.to_string_lossy().to_string(),
                self.status,
            ))
            .attach_printable(format!("running {}", self.child.get_command_string()))
        }
    }
}
