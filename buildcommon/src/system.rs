//! Common system operations
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, ChildStdin, ChildStdout, ExitStatus, Stdio};

use error_stack::{Result, ResultExt};
use filetime::FileTime;

use crate::{errorln, verboseln};

/// Error messages
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("failed to initialize environment")]
    InitEnv,
    #[error("cannot find megaton repository root! bad setup?")]
    FindToolRoot,
    #[error("cannot find megaton project root")]
    FindProjectRoot,

    // === path operations ===
    #[error("failed to get current executable path")]
    CurrentExe,
    #[error("getting parent path of root")]
    ParentPath,
    #[error("failed to canonicalize `{0}`")]
    Canonicalize(String),

    // === file operations ===
    #[error("failed to read from `{0}`")]
    ReadFile(String),
    #[error("failed to read YAML from `{0}`")]
    ReadYaml(String),
    #[error("failed to write to `{0}`")]
    WriteFile(String),
    #[error("failed to get modified time for `{0}`")]
    GetMTime(String),
    #[error("failed to set modified time for `{0}`")]
    SetMTime(String),
    #[error("failed to remove directory `{0}`")]
    RemoveDirectory(String),
    #[error("failed to create directory `{0}`")]
    CreateDirectory(String),

    // === process operations ===
    #[error("failed to spawn `{0}`")]
    Spawn(String),
    #[error("failed to execute `{0}`")]
    Subcommand(String),
    
}

/// Path extensions
pub trait PathExt: Sized + AsRef<Path> {
    /// Get the parent path, create a report on error
    fn parent_or_err(&self) -> Result<&Path, Error>;
    /// Get the parent path, create a report on error
    fn into_parent(self) -> Result<PathBuf, Error>;
    /// Push `path` onto `self` and return the result
    fn into_joined(self, path: impl AsRef<Path>) -> PathBuf;
    /// Convert to an absolute path
    fn to_abs(&self) -> Result<PathBuf, Error> {
        let path = self.as_ref();
        dunce::canonicalize(path)
            .change_context_lazy(|| Error::Canonicalize(path.display().to_string()))
    }
    /// Convert to relative path from base.
    ///
    /// Both self and base should be absolute
    fn rebase(&self, base: impl AsRef<Path>) -> PathBuf {
        let path = self.as_ref();
        let base = base.as_ref();
        assert!(path.is_absolute());
        assert!(base.is_absolute());
        pathdiff::diff_paths(path, base).unwrap_or(path.to_path_buf())
    }
}

impl PathExt for PathBuf {
    fn parent_or_err(&self) -> Result<&Path, Error> {
        Ok(self.parent().ok_or(Error::ParentPath)?)
    }

    fn into_parent(mut self) -> Result<PathBuf, Error> {
        let ok = self.pop();
        if !ok {
            Err(Error::ParentPath)?;
        }
        Ok(self)
    }

    #[inline]
    fn into_joined(mut self, path: impl AsRef<Path>) -> PathBuf {
        self.push(path);
        self
    }
}

impl PathExt for &Path {
    fn parent_or_err(&self) -> Result<&Path, Error> {
        Ok(self.parent().ok_or(Error::ParentPath)?)
    }

    fn into_parent(self) -> Result<PathBuf, Error> {
        Ok(self.parent_or_err()?.to_path_buf())
    }

    fn into_joined(self, path: impl AsRef<Path>) -> PathBuf {
        self.join(path)
    }
}

/// Create file for buffered writing
pub fn buf_writer(path: impl AsRef<Path>) -> Result<BufWriter<File>, Error>
{
    let path = path.as_ref();
    let file = File::create(path)
        .change_context_lazy(|| Error::WriteFile(path.display().to_string()))?;
    Ok(BufWriter::new(file))
}

/// Open file for buffered reading
pub fn buf_reader(path: impl AsRef<Path>) -> Result<BufReader<File>, Error>
{
    let path = path.as_ref();
    let file = 
    File::open(path)
        .change_context_lazy(|| Error::ReadFile(path.display().to_string()))?;
    Ok(BufReader::new(file))
}

/// Write content to a file
pub fn write_file(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<(), Error>
{
    let path = path.as_ref();
    std::fs::write(path, content)
        .change_context_lazy(|| Error::WriteFile(path.display().to_string()))
}

/// Read file as string
pub fn read_file(path: impl AsRef<Path>) -> Result<String, Error>
{
    let path = path.as_ref();
    std::fs::read_to_string(path)
        .change_context_lazy(|| Error::ReadFile(path.display().to_string()))
}

/// Get the modified time for a file.
///
/// If the file doesn't exist, None is returned
pub fn get_mtime(path: impl AsRef<Path>) -> Result<Option<FileTime>, Error>
{
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None)
    }

    let metadata = path.metadata()
        .change_context_lazy(|| Error::GetMTime(path.display().to_string()))?;

    Ok(Some(FileTime::from_last_modification_time(&metadata)))
}

/// Set the modified time for a file
pub fn set_mtime(path: impl AsRef<Path>, time: FileTime) -> Result<(), Error>
{
    let path = path.as_ref();
    filetime::set_file_mtime(path, time)
        .change_context_lazy(|| Error::SetMTime(path.display().to_string()))
}

/// Return true if the build edge in -> out is up-to-date
#[inline]
pub fn up_to_date(in_mtime: Option<FileTime>, out_mtime: Option<FileTime>) -> bool
{
    match (in_mtime, out_mtime) {
        (Some(in_mtime), Some(out_mtime)) => in_mtime < out_mtime,
        _ => false,
    }
}

/// Convenience wrapper for std::fs::remove_dir_all
pub fn remove_directory(path: impl AsRef<Path>) -> Result<(), Error>
{
    let path = path.as_ref();
    verboseln!("rm -r '{}'", path.display());
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(path)
        .change_context_lazy(|| Error::RemoveDirectory(path.display().to_string()))
}

/// Convenience wrapper for std::fs::create_dir_all
pub fn ensure_directory(path: impl AsRef<Path>) -> Result<(), Error>
{
    let path = path.as_ref();
    if path.exists() {
        return Ok(());
    }
    verboseln!("mkdir -p '{}'", path.display());
    std::fs::create_dir_all(path)
        .change_context_lazy(|| Error::CreateDirectory(path.display().to_string()))

}
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
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self
    {
        self.command.current_dir(dir);
        self
    }

    /// Set args as in `Command`
    #[inline]
    pub fn args(mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Self
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

    pub fn spawn(mut self) -> Result<Spawned, Error> {
        verboseln!("running {}", self.get_command_string());
        // we don't care about escaping it properly, just for debugging
        let child = self
            .command
            .spawn()
            .change_context_lazy(|| Error::Spawn(self.executable.to_string_lossy().to_string()))?;

        Ok(Spawned { command: self, child })
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
            .change_context_lazy(|| Error::Subcommand(
                self.command.executable.to_string_lossy().to_string()
            ))
                .attach_printable_lazy(|| format!("running {}", self.get_command_string()))
        ?;

        Ok(Finished { status, child: self })
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
}
