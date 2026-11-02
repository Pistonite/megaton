
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use cu::pre::*;

/// Clang's JSON Compilation Database manager
///
/// We use an extra field to also store clangd-compatible arguments
/// (since our main compiler is DKP's), and clangd-compatible arguments
/// are saved to user-supplied compile_commands.json location.
pub struct CompileDatabase {
    // see https://clang.llvm.org/docs/JSONCompilationDatabase.html

    /// If the in-memory map is different from the file on the disk
    dirty: bool,
    /// Path to save the database
    path: PathBuf,
    /// Compile commands data
    commands: BTreeMap<String, CompileCommand>,
}

impl CompileDatabase {
    #[cu::context("failed to load compile database from {}", path.display())]
    pub fn load(path: &Path) -> cu::Result<Self> {
        let raw_commands = {
            let reader = cu::fs::reader(path)?;
            json::read::<Vec<CompileCommand>>(reader)?
        };
        let old_len = raw_commands.len();
        let commands = raw_commands.into_iter().filter_map(|command| {
            if command.file.is_empty() || command.output.is_empty() {
                return None;
            }
            if command.arguments.is_empty() && command.command.is_none() {
                return None;
            }
            Some((command.file.clone(), command))
        }).collect::<BTreeMap<_,_>>();

        let dirty = old_len != commands.len();

        Ok(Self {
            dirty, path: path.to_path_buf(), commands
        })
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[cu::context("failed to save compile database to {}", self.path.display())]
    pub fn save(&self) -> cu::Result<()> {
        use serde::ser::SerializeSeq as _;
        use serde::Serializer as _;
        use std::io::Write as _;

        if !self.dirty {
            cu::debug!("compile database is up-to-date, skipping serializing");
            return Ok(());
        }

        let mut writer  = cu::fs::buf_writer(&self.path)?;
            let mut ser = json::ser::Serializer::pretty(&mut writer);
            let mut ser_seq = ser.serialize_seq(None)?;
            for command in self.commands.values() {
                ser_seq.serialize_element(&command)?;
            }
            ser_seq.end()?;
            writer.flush()?;

        cu::debug!("compile database saved to {}", self.path.display());
        Ok(())
    }

    /// Save the compile database to the path, merging with other files
    /// already in there, and serialize clangd compatible arguments
    #[cu::context("failed to save clangd-compatible compile database to {}", path.display())]
    pub fn save_clangd_compat_merge_with(&self, path: &Path) -> cu::Result<()> {
        use serde::ser::SerializeSeq as _;
        use serde::Serializer as _;
        use std::io::Write as _;

        let mut clangd_compdb = Self::load(path)?;
        for command in self.commands.values() {
            clangd_compdb.update_clangd_compat(Cow::Borrowed(command));
        }

        if clangd_compdb.dirty {
            let mut writer  = cu::fs::buf_writer(&self.path)?;
            let mut ser = json::ser::Serializer::pretty(&mut writer);
            let mut ser_seq = ser.serialize_seq(None)?;
            for command in clangd_compdb.commands.values() {
                ser_seq.serialize_element(&ClangdCompatCompileCommandSer(command))?;
            }
            ser_seq.end()?;
            writer.flush()?;
            drop(writer);

            cu::debug!("saved clangd-compatible compile database is up-to-date");
        } else {
            cu::debug!("clangd-compatible compile database is up-to-date, skipping save");
        }

        Ok(())
    }

    pub fn get(&self, file: &str) -> Option<&CompileCommand> {
        self.commands.get(file)
    }

    pub fn update(&mut self, command: Cow<'_, CompileCommand>) {
        self.update_imp(command, |a, b| a == b);
    }

    fn update_clangd_compat(&mut self, command: Cow<'_, CompileCommand>) {
        self.update_imp(command, eq_clangd_compat);
    }

    #[inline(always)]
    fn update_imp<F: Fn(&CompileCommand, &CompileCommand)->bool>(&mut self, command: Cow<'_, CompileCommand>, eq_fn: F) {
        use std::collections::btree_map::Entry;
        match self.commands.entry(command.file.clone()) {
            Entry::Vacant(e) => {
                e.insert(command.into_owned());
                self.dirty = true;
            }
            Entry::Occupied(mut e) => {
                if eq_fn(e.get(), command.as_ref()) {
                    return;
                }
                e.insert(command.into_owned());
                self.dirty = true;
            }
        }
    }

}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompileCommand {
    // see https://clang.llvm.org/docs/JSONCompilationDatabase.html
    
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub directory: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub output: String,

    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clangd_compat_arguments: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>
}

fn eq_clangd_compat(a: &CompileCommand, b: &CompileCommand) -> bool {
    if a.directory != b.directory {
        return false;
    }
    if a.file != b.file {
        return false;
    }
    if a.output != b.output {
        return false;
    }
    if !a.clangd_compat_arguments.is_empty() {
        if a.clangd_compat_arguments == b.arguments {
            return true;
        }
    }
    if !b.clangd_compat_arguments.is_empty() {
        if b.clangd_compat_arguments == a.arguments {
            return true;
        }
    }
    false
}

struct ClangdCompatCompileCommandSer<'a>(&'a CompileCommand);
impl Serialize for ClangdCompatCompileCommandSer<'_> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error>{
        serialize_clangd_compat(self.0, ser)
    }
}

fn serialize_clangd_compat<S: serde::Serializer>(command: &CompileCommand, ser: S)
    -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeStruct as _;

    let arguments = if !command.clangd_compat_arguments.is_empty() {
        Some(&command.clangd_compat_arguments)
    } else if !command.arguments.is_empty() {
        Some(&command.arguments)
    } else {
        None
    };
    // command is only used as the fallback when there are no arguments
    let shell_command = match arguments {
        Some(_) => None,
        None => command.command.as_ref(),
    };

    let len = 2 // file, output
        + usize::from(!command.directory.is_empty())
        + usize::from(arguments.is_some())
        + usize::from(shell_command.is_some());

    let mut s = ser.serialize_struct("CompileCommand", len)?;
    if !command.directory.is_empty() {
        s.serialize_field("directory", &command.directory)?;
    }
    s.serialize_field("file", &command.file)?;
    s.serialize_field("output", &command.output)?;
    if let Some(arguments) = arguments {
        s.serialize_field("arguments", arguments)?;
    }
    if let Some(shell_command) = shell_command {
        s.serialize_field("command", shell_command)?;
    }
    s.end()
}
