//! Compile DB and compile_commands.json related functionality.
use crate::env::SystemHeaderPaths;
use crate::{env::Env, prelude::*};

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::Path;

use derive_more::derive::{Deref, DerefMut};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::system::Command;

/// Compile DB format used internally by build tool to track
/// if command changed and if objects needs to be rebuilt.
///
/// The arguments do not include system header include paths
/// (i.e. are what's actually passed to the compiler).
#[derive(Debug, Default, Serialize, Deserialize, Deref, DerefMut)]
pub struct CompileDB {
    /// Map of output file path to compile command
    entries: FxHashMap<String, CompileDBEntry>,
}

impl CompileDB {
    /// Load compile DB from file. If file does not exist or there is IO error, return empty or
    /// partial DB.
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        verboseln!("loading '{}'", path.display());
        if !path.exists() {
            return Self::default();
        }
        let file = match system::buf_reader(path) {
            Ok(file) => file,
            Err(_) => {
                return Self::default();
            }
        };
        serde_json::from_reader(file).unwrap_or_default()
    }

    /// Save the Compile DB and compile_commands.json. If fails, errors are logged to console
    pub fn save(
        &self,
        system_headers: &SystemHeaderPaths,
        path: impl AsRef<Path>,
        cc_json: impl AsRef<Path>,
    ) {
        let path = path.as_ref();
        verboseln!("saving '{}'", path.display());
        let file = match system::buf_writer(path) {
            Ok(file) => file,
            Err(e) => {
                errorln!(
                    "Error",
                    "Failed to save Compile DB: failed to open file: {}",
                    e
                );
                return;
            }
        };
        if let Err(e) = serde_json::to_writer_pretty(file, self) {
            errorln!("Error", "Failed to save Compile DB: {}", e);
        }
        let cc_json = cc_json.as_ref();
        verboseln!("saving '{}'", cc_json.display());

        let mut file = match system::buf_writer(cc_json) {
            Ok(file) => file,
            Err(e) => {
                errorln!(
                    "Error",
                    "Failed to save compile_commands.json: failed to open file: {}",
                    e
                );
                return;
            }
        };

        if let Err(e) = self.write_as_compile_commands(system_headers, &mut file) {
            errorln!("Error", "Failed to save compile_commands.json: {}", e);
        }
    }

    /// Merge with old compile DB. If newer command exists, keep the new one (the one currently in
    /// self)
    pub fn merge_old(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let mut old = Self::load(path);
        std::mem::swap(self, &mut old);
        self.entries.extend(old.entries);
    }

    /// Write the compile_commands.json file
    pub fn write_as_compile_commands<W: Write>(
        &self,
        sh: &SystemHeaderPaths,
        write: &mut W,
    ) -> std::io::Result<()> {
        write!(write, "[")?;
        let mut iter = self.entries.iter();
        if let Some((output, entry)) = iter.next() {
            entry.write_as_compile_command(sh, output, write)?;
            for (output, entry) in &self.entries {
                writeln!(write, ",")?;
                entry.write_as_compile_command(sh, output, write)?;
            }
        }
        writeln!(write, "]")
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CompileDBEntry {
    /// Arguments up to the "-c -o OUTPUT FILE"
    pub args: Vec<String>,
    /// Input file
    pub file: String,
}

impl CompileDBEntry {
    /// Create a new compdb entry
    pub fn new(
        compiler: impl AsRef<Path>,
        source_file: &str,
        dep_file: String,
        args: impl IntoIterator<Item = String>,
    ) -> Self {
        let args = std::iter::once(compiler.as_ref().display().to_string())
            .chain([
                "-MMD".to_string(),
                "-MP".to_string(),
                "-MF".to_string(),
                dep_file,
            ])
            .chain(args)
            .collect();
        Self {
            args,
            file: source_file.to_string(),
        }
    }
    /// Create a child command for compiling this file to output
    pub fn create_command(&self, output: impl AsRef<OsStr>) -> Command {
        Command::new(&self.args[0])
            .args(self.args[1..].iter().map(|s| s.as_ref()).chain([
                OsStr::new("-c"),
                "-o".as_ref(),
                output.as_ref(),
                self.file.as_ref(),
            ]))
            .silence_stdout()
            .pipe_stderr()
    }
    /// Write the entry as a compile_commands.json entry (no trailing comma)
    pub fn write_as_compile_command<W: Write>(
        &self,
        sh: &SystemHeaderPaths,
        output: &str,
        write: &mut W,
    ) -> std::io::Result<()> {
        let file_escaped = self.file.replace("\\", "\\\\").replace("\"", "\\\"");
        let output_escaped = output.replace("\\", "\\\\").replace("\"", "\\\"");
        // the paths are all absolute, so directory is just a placeholder
        // (it's required in compile_commands.json)
        write!(
            write,
            r#"{{"directory":"/",
"file":"{}",
"output":"{}",
"command":""#,
            file_escaped, output_escaped
        )?;
        for arg in &self.args {
            let arg = arg.replace("\\", "\\\\").replace("\"", "\\\"");
            write!(write, "{} ", arg)?;
        }
        // inject system headers for clangd
        sh.write(&self.file, false, write)?;
        write!(write, "-c -o {} {}\"}}", output_escaped, file_escaped)
    }
}

/// One command in compile_commands.json
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompileCommand {
    pub directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    pub file: String,
    pub output: String,
}

/// Read compile_commands.json file from reader, inject system headers and write to writer
///
/// This is for injecting system includes into ninja compdb output
pub fn inject_system_headers<R: Read, W: Write>(
    env: &Env,
    read: R,
    write: &mut W,
) -> std::io::Result<()> {
    let mut commands: Vec<CompileCommand> = serde_json::from_reader(read)?;
    for command in commands.iter_mut() {
        if let Some(cmd) = &mut command.command {
            env.system_headers.add_to_command_str(&command.file, cmd);
        }
        if let Some(arguments) = &mut command.arguments {
            env.system_headers
                .add_to_command_vec(&command.file, arguments);
        }
    }
    serde_json::to_writer_pretty(write, &commands)?;
    Ok(())
}
