//! Subprocess Utilities
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

use crate::system::Error;
use buildcommon::errorln;

/// Convenience macro for building an argument list
macro_rules! args {
    ($($arg:expr),* $(,)?) => {
        {
            let args: Vec<&std::ffi::OsStr> = vec![$($arg.as_ref()),*];
            args
        }
    };
}
pub(crate) use args;

/// Convenience wrapper around `Command` for building a child process
pub struct ChildBuilder {
    arg0: String,
    command: Command,
}

impl ChildBuilder {
    pub fn new<S>(arg0: S) -> Self
    where
        S: AsRef<OsStr>,
    {
        Self {
            arg0: arg0.as_ref().to_string_lossy().to_string(),
            command: Command::new(arg0),
        }
    }

    #[inline]
    pub fn current_dir<P>(mut self, dir: P) -> Self
    where
        P: AsRef<Path>,
    {
        self.command.current_dir(dir);
        self
    }

    /// Set args as in `Command`
    #[inline]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
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

    pub fn spawn(mut self) -> Result<ChildProcess, Error> {
        // we don't care about escaping it properly, just for debugging
        let args_str = self
            .command
            .get_args()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let command_str = format!("{} {}", self.arg0, args_str);
        let child = self
            .command
            .spawn()
            .map_err(|e| Error::SpawnChild(command_str.clone(), e))?;
        Ok(ChildProcess { command_str, child })
    }
}

/// Convenience wrapper around `Child` for a spawned process
pub struct ChildProcess {
    command_str: String,
    child: Child,
}

impl ChildProcess {
    pub fn command(&self) -> &str {
        &self.command_str
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
    pub fn wait(mut self) -> Result<ExitStatus, Error> {
        let status = self
            .child
            .wait()
            .map_err(|e| Error::WaitForChild(self.command_str.clone(), e))?;
        Ok(status)
    }

    /// Take the stderr, and dump it using `errorln!`
    pub fn dump_stderr(&mut self, prefix: &str) {
        if let Some(stderr) = self.take_stderr() {
            for line in stderr.lines().map_while(Result::ok) {
                errorln!(prefix, "{}", line);
            }
        }
    }
}
