//! Build flags processing

use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::ChildStderr;

use buildcommon::flags::Flags;
use buildcommon::system::PathExt;
use buildcommon::{system, verboseln};
use error_stack::Result;
use rustc_hash::{FxHashMap, FxHasher};
use serde::{Deserialize, Serialize};

use crate::build::config::Build;
use crate::build::Paths;
use crate::system::{ChildBuilder, ChildProcess};

pub struct Builder<'a> {
    paths: &'a Paths,
    flags: Flags,
}

impl<'a> Builder<'a> {
    pub fn new(paths: &'a Paths, entry: &str, build: &Build) -> Result<Self, system::Error> {
        let mut flags = Flags::from_config(&build.flags);

        let mut includes = Vec::with_capacity(build.includes.len() + 1);
        includes.push(format!("-I{}", paths.libnx_include.display()));
        for dir in &build.includes {
            let path = paths.root.join(dir).to_abs()?;
            includes.push(format!("-I{}", path.display()));
        }

        flags.add_includes(includes);
        flags.set_init(entry);
        flags.set_version_script(paths.verfile.display());
        flags.add_libpaths(
            build
                .libpaths
                .iter()
                .map(|libpath| paths.root.join(libpath).to_abs())
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .map(|path| path.display()),
        );
        flags.add_libraries(&build.libraries);
        flags.add_ldscripts(
            build
                .ldscripts
                .iter()
                .map(|ldscript| paths.root.join(ldscript).to_abs())
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .map(|path| path.display()),
        );

        Ok(Self { paths, flags })
    }

    fn create_command(
        &self,
        s_type: SourceType,
        source: String,
        d_file: String,
        o_file: String,
    ) -> CompileCommand {
        let arguments: Vec<_> = match s_type {
            SourceType::C => std::iter::once(self.paths.make_c.display().to_string())
                .chain([
                    "-MMD".to_string(),
                    "-MP".to_string(),
                    "-MF".to_string(),
                    d_file,
                ])
                .chain(self.flags.cflags.iter().cloned())
                .chain([
                    "-c".to_string(),
                    "-o".to_string(),
                    o_file.clone(),
                    source.clone(),
                ])
                .collect(),
            SourceType::Cpp => std::iter::once(self.paths.make_cpp.display().to_string())
                .chain([
                    "-MMD".to_string(),
                    "-MP".to_string(),
                    "-MF".to_string(),
                    d_file,
                ])
                .chain(self.flags.cxxflags.iter().cloned())
                .chain([
                    "-c".to_string(),
                    "-o".to_string(),
                    o_file.clone(),
                    source.clone(),
                ])
                .collect(),
            SourceType::S => std::iter::once(self.paths.make_cpp.display().to_string())
                .chain([
                    "-MMD".to_string(),
                    "-MP".to_string(),
                    "-MF".to_string(),
                    d_file,
                    "-x".to_string(),
                    "assembler-with-cpp".to_string(),
                ])
                .chain(self.flags.sflags.iter().cloned())
                .chain([
                    "-c".to_string(),
                    "-o".to_string(),
                    o_file.clone(),
                    source.clone(),
                ])
                .collect(),
        };

        CompileCommand {
            directory: "/".to_string(),
            arguments,
            file: source,
            output: o_file,
        }
    }

    pub fn process_source(
        &self,
        source_path: &Path,
        cc_possibly_changed: bool,
        compile_commands: &mut FxHashMap<String, CompileCommand>,
    ) -> Result<SourceResult, system::Error> {
        let source = source_path.display().to_string();
        let (source_type, base, ext) = match get_source_type(&source) {
            Some(x) => x,
            None => {
                return Ok(SourceResult::NotSource);
            }
        };
        let hashed = source_hashed(&source, base, ext);
        let o_path = self.paths.target_o.join(&format!("{}.o", hashed));
        let o_file = o_path.display().to_string();
        let d_path = self.paths.target_o.join(&format!("{}.d", hashed));
        let d_file = d_path.display().to_string();
        if !o_path.exists() {
            // output doesn't exist
            let cc = self.create_command(source_type, source, d_file, o_file);
            return Ok(SourceResult::NeedCompile(cc));
        }
        // d file changed? (source included in d_file)
        if !are_deps_up_to_date(&d_path, &o_path)? {
            let cc = self.create_command(source_type, source, d_file, o_file);
            return Ok(SourceResult::NeedCompile(cc));
        }
        // dependencies didn't change. Only rebuild if compile command changed
        if !cc_possibly_changed {
            return Ok(SourceResult::UpToDate(o_file));
        }
        let cc = self.create_command(source_type, source, d_file, o_file);
        match compile_commands.remove(&cc.output) {
            Some(old_cc) => {
                if old_cc == cc {
                    Ok(SourceResult::UpToDate(cc.output))
                } else {
                    Ok(SourceResult::NeedCompile(cc))
                }
            }
            None => {
                // no previous command found, (never built), need build
                Ok(SourceResult::NeedCompile(cc))
            }
        }
    }

    pub fn link_start(&self, objects: &[String], elf: &Path) -> std::result::Result<BuildTask, crate::system::Error> {
        // use CXX for linking
        let child = ChildBuilder::new(&self.paths.make_cpp)
            .args(
                self.flags
                    .ldflags
                    .iter()
                    .chain(objects.iter())
                    .chain(["-o".to_string(), elf.display().to_string()].iter()),
            )
            .silence_stdout()
            .pipe_stderr()
            .spawn()?;
        verboseln!("running {}", child.command());
        Ok(BuildTask { child })
    }
}

pub enum SourceResult {
    NotSource,
    UpToDate(String),
    NeedCompile(CompileCommand),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompileCommand {
    #[serde(default)]
    directory: String,
    pub arguments: Vec<String>,
    pub file: String,
    pub output: String,
}

pub struct LinkResult {
    pub success: bool,
    pub error: Vec<String>,
}

impl CompileCommand {
    pub fn create_child(&self) -> ChildBuilder {
        ChildBuilder::new(&self.arguments[0])
            .args(&self.arguments[1..])
            .silence_stdout()
            .pipe_stderr()
    }
}

pub struct BuildTask {
    child: ChildProcess,
}

impl BuildTask {
    pub fn new(child: ChildProcess) -> Self {
        verboseln!("running {}", child.command());
        Self { child }
    }
    pub fn wait(self) -> std::result::Result<BuildResult, crate::system::Error> {
        let mut child = self.child;
        let error = child.take_stderr();
        let status = child.wait()?;
        Ok(BuildResult {
            success: status.success(),
            error,
        })
    }
}

pub struct BuildResult {
    pub success: bool,
    pub error: Option<BufReader<ChildStderr>>,
}

pub fn load_compile_commands(cc_json: &Path, map: &mut FxHashMap<String, CompileCommand>) {
    verboseln!("loading '{}'", cc_json.display());
    if !cc_json.exists() {
        return;
    }
    let file = match system::buf_reader(cc_json) {
        Ok(file) => file,
        _ => {
            return;
        }
    };
    let ccs: Vec<CompileCommand> = match serde_json::from_reader(file) {
        Ok(ccs) => ccs,
        Err(_) => return,
    };
    for cc in ccs {
        map.insert(cc.output.clone(), cc);
    }
}

pub enum SourceType {
    C,
    Cpp,
    S,
}

impl SourceType {
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            ".c" => Some(Self::C),
            ".cpp" | ".cc" | ".cxx" | ".c++" => Some(Self::Cpp),
            ".s" | ".asm" => Some(Self::S),
            _ => None,
        }
    }
}

fn get_source_type(source: &str) -> Option<(SourceType, &str, &str)> {
    let dot = source.rfind('.').unwrap_or(source.len());
    let ext = &source[dot..];
    let source_type = SourceType::from_ext(ext)?;
    let slash = source.rfind(|c| c == '/' || c == '\\').unwrap_or(0);
    let base = &source[slash + 1..dot];
    if base.is_empty() {
        return None;
    }
    Some((source_type, base, ext))
}

fn source_hashed(source: &str, base: &str, ext: &str) -> String {
    let mut hasher = FxHasher::default();
    source.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{}-{:016x}{}", base, hash, ext)
    // format!("{}{}", base, ext)
}

fn are_deps_up_to_date(d_path: &Path, o_path: &Path) -> Result<bool, system::Error> {
    if !d_path.exists() {
        return Ok(false);
    }
    // (very strong) assumptions of the depfiles:
    // - the first rule is what we care about (the target)
    // - the first line is just the target
    let o_mtime = system::get_mtime(o_path)?;
    let file = system::buf_reader(d_path)?;
    let lines = file.lines();
    for line in lines.skip(1) {
        // skip the <target>: \ line
        let line = match line {
            Ok(x) => x,
            Err(_) => return Ok(false),
        };
        let part = line.trim().trim_end_matches('\\').trim_end();
        if part.ends_with(':') {
            break;
        }
        let d_mtime = system::get_mtime(part)?;
        if !system::up_to_date(d_mtime, o_mtime) {
            return Ok(false);
        }
    }
    Ok(true)
}
