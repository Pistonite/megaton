use std::process::ExitStatus;

use error_stack::{report, Report};

/// Error messages
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("failed to initialize environment")]
    InitEnv,
    #[error("cannot find megaton repository root! bad setup?")]
    FindToolRoot,
    #[error("cannot find megaton project root")]
    FindProjectRoot,
    #[error("expect: {0}")]
    Expect(&'static str),

    // === path operations ===
    #[error("getting parent path of root")]
    ParentPath,
    #[error("failed to canonicalize `{0}`")]
    Canonicalize(String),
    #[error("path should be utf-8: {0}")]
    NotUTF8(String),

    // === file operations ===
    #[error("failed to read from `{0}`")]
    ReadFile(String),
    #[error("failed to read YAML from `{0}`")]
    ReadYaml(String),
    #[error("failed to write to `{0}`")]
    WriteFile(String),
    #[error("failed to remove file `{0}`")]
    RemoveFile(String),
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
    #[error("{0} exited with status: {1}")]
    ExitStatus(String, ExitStatus),
}

/// Marker trait for errors that can be used
/// in the context wrapper system
pub trait Context: error_stack::Context {}

/// Trait for wrapping execution with some context
pub trait ChangeContext: Sized {
    type Target: error_stack::Context;
    fn change_context(report: Report<impl Context>) -> Report<Self::Target>;
}

/// Wrapper for Report so we can implement our own traits
#[repr(transparent)]
pub struct ReportWrapper<CC: ChangeContext>(Report<CC::Target>);

/// A Result type that can be used to wrap errors with context
/// automatically when using the `?` operator
pub type ResultIn<T, C> = Result<T, ReportWrapper<C>>;

/// Implementation for converting an error to report wrapper with
/// the `?` operator
impl<E: Context, CC: ChangeContext> From<E> for ReportWrapper<CC> {
    #[track_caller]
    fn from(value: E) -> Self {
        Self(CC::change_context(report!(value)))
    }
}

/// Implementation for converting a Report to report wrapper with
/// the `?` operator
impl<E: Context, CC: ChangeContext> From<Report<E>> for ReportWrapper<CC> {
    #[track_caller]
    fn from(value: Report<E>) -> Self {
        Self(CC::change_context(value))
    }
}

/// Implementation for converting a ReportWrapper to back to a Report
impl<CC: ChangeContext> From<ReportWrapper<CC>> for Report<CC::Target> {
    fn from(value: ReportWrapper<CC>) -> Report<CC::Target> {
        value.0
    }
}

/// Create a type and implement the ChangeContext trait for it
#[macro_export]
macro_rules! error_context {
    ($ty:ident, | $report:ident | -> $target:ty $body:block) => {
        struct $ty;
        impl $crate::system::ChangeContext for $ty {
            type Target = $target;
            #[inline]
            fn change_context($report: error_stack::Report<impl $crate::system::Context>) -> error_stack::Report<$target> {
                $body
            }
        }
    };
    ($vis:vis $ty:ident, | $report:ident | -> $target:ty $body:block) => {
        $vis struct $ty;
        impl $crate::system::ChangeContext for $ty {
            type Target = $target;
            #[inline]
            fn change_context($report: error_stack::Report<impl $crate::system::Context>) -> error_stack::Report<$target> {
                $body
            }
        }
    }
}

/*
# MIT License

Copyright © 2022–, HASH

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/// See [`ResultExt`] for more information.
pub trait ResultInExt {
    /// The [`Context`] type of the [`Result`].
    type Context: error_stack::Context;

    /// Type of the [`Ok`] value in the [`Result`]
    type Ok;

    /// Adds a new attachment to the [`Report`] inside the [`Result`].
    ///
    /// Applies [`Report::attach`] on the [`Err`] variant, refer to it for more information.
    fn attach<A>(self, attachment: A) -> Result<Self::Ok, Report<Self::Context>>
    where
        A: Send + Sync + 'static;

    /// Lazily adds a new attachment to the [`Report`] inside the [`Result`].
    ///
    /// Applies [`Report::attach`] on the [`Err`] variant, refer to it for more information.
    fn attach_lazy<A, F>(self, attachment: F) -> Result<Self::Ok, Report<Self::Context>>
    where
        A: Send + Sync + 'static,
        F: FnOnce() -> A;

    /// Adds a new printable attachment to the [`Report`] inside the [`Result`].
    ///
    /// Applies [`Report::attach_printable`] on the [`Err`] variant, refer to it for more
    /// information.
    fn attach_printable<A>(self, attachment: A) -> Result<Self::Ok, Report<Self::Context>>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static;

    /// Lazily adds a new printable attachment to the [`Report`] inside the [`Result`].
    ///
    /// Applies [`Report::attach_printable`] on the [`Err`] variant, refer to it for more
    /// information.
    fn attach_printable_lazy<A, F>(
        self,
        attachment: F,
    ) -> core::result::Result<Self::Ok, Report<Self::Context>>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> A;

    /// Changes the context of the [`Report`] inside the [`Result`].
    ///
    /// Applies [`Report::change_context`] on the [`Err`] variant, refer to it for more information.
    fn change_context<C>(self, context: C) -> Result<Self::Ok, Report<C>>
    where
        C: Context;

    /// Lazily changes the context of the [`Report`] inside the [`Result`].
    ///
    /// Applies [`Report::change_context`] on the [`Err`] variant, refer to it for more information.
    fn change_context_lazy<C, F>(self, context: F) -> Result<Self::Ok, Report<C>>
    where
        C: Context,
        F: FnOnce() -> C;
}

impl<T, C> ResultInExt for ResultIn<T, C>
where
    C: ChangeContext,
    <C as ChangeContext>::Target: std::error::Error + error_stack::Context,
{
    type Context = C::Target;
    type Ok = T;

    #[track_caller]
    fn attach<A>(self, attachment: A) -> Result<T, Report<Self::Context>>
    where
        A: Send + Sync + 'static,
    {
        // Can't use `map_err` as `#[track_caller]` is unstable on closures
        match self {
            Ok(ok) => Ok(ok),
            Err(ReportWrapper(report)) => Err(report.attach(attachment)),
        }
    }

    #[track_caller]
    fn attach_lazy<A, F>(self, attachment: F) -> Result<T, Report<Self::Context>>
    where
        A: Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        // Can't use `map_err` as `#[track_caller]` is unstable on closures
        match self {
            Ok(ok) => Ok(ok),
            Err(ReportWrapper(report)) => Err(report.attach(attachment())),
        }
    }

    #[track_caller]
    fn attach_printable<A>(self, attachment: A) -> Result<T, Report<Self::Context>>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        // Can't use `map_err` as `#[track_caller]` is unstable on closures
        match self {
            Ok(ok) => Ok(ok),
            Err(ReportWrapper(report)) => Err(report.attach_printable(attachment)),
        }
    }

    #[track_caller]
    fn attach_printable_lazy<A, F>(self, attachment: F) -> Result<T, Report<Self::Context>>
    where
        A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> A,
    {
        // Can't use `map_err` as `#[track_caller]` is unstable on closures
        match self {
            Ok(ok) => Ok(ok),
            Err(ReportWrapper(report)) => Err(report.attach_printable(attachment())),
        }
    }

    #[track_caller]
    fn change_context<C2>(self, context: C2) -> Result<T, Report<C2>>
    where
        C2: Context,
    {
        // Can't use `map_err` as `#[track_caller]` is unstable on closures
        match self {
            Ok(ok) => Ok(ok),
            Err(ReportWrapper(report)) => Err(report.change_context(context)),
        }
    }

    #[track_caller]
    fn change_context_lazy<C2, F>(self, context: F) -> Result<T, Report<C2>>
    where
        C2: Context,
        F: FnOnce() -> C2,
    {
        // Can't use `map_err` as `#[track_caller]` is unstable on closures
        match self {
            Ok(ok) => Ok(ok),
            Err(ReportWrapper(report)) => Err(report.change_context(context())),
        }
    }
}

// while not idea, only this crate can implement the Context trait
// so we need to implement all foreign errors here

impl Context for Error {}
impl Context for std::io::Error {}
impl Context for regex::Error {}
impl Context for toml::de::Error {}
impl Context for serde_json::Error {}
