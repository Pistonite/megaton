//! Build flags processing
use buildcommon::prelude::*;

use std::io::BufRead;
use std::path::{Path, PathBuf};

use filetime::FileTime;

use buildcommon::compdb::{CompileDB, CompileDBEntry};
use buildcommon::env::ProjectEnv;
use buildcommon::flags::Flags;
use buildcommon::source::{SourceFile, SourceType};
use buildcommon::system::{Command, Spawned};

use crate::error::Error;

use super::config::{Build, Module};

pub struct Builder<'a> {
    env: &'a ProjectEnv,
    flags: Flags,
    pub source_dirs: Vec<PathBuf>,
    lib_objects: Vec<String>,
}

error_context!(pub BuilderNew, |r| -> Error {
    errorln!("Failed", "Preparing build");
    r.change_context(Error::BuildPrep)
});
impl<'a> Builder<'a> {
    pub fn new(env: &'a ProjectEnv, module: &Module, build: &Build) -> ResultIn<Self, BuilderNew> {
        build.check()?;
        let mut flags = Flags::from_config(&build.flags);
        let mut source_dirs = Vec::with_capacity(build.sources.len() + 1);

        if build.libmegaton {
            let lib_path = env.megaton_home.join("lib");

            flags.add_includes([lib_path.join("include").display()]);

            let runtime_source = lib_path.into_joined("runtime");

            source_dirs.push(runtime_source);

            let defines = [
                format!("MEGART_NX_MODULE_NAME=\"{}\"", module.name),
                format!("MEGART_NX_MODULE_NAME_LEN={}", module.name.len()),
                format!("MEGART_TITLE_ID={}", module.title_id),
                format!("MEGART_TITLE_ID_HEX=\"{}\"", module.title_id_hex()),
            ];

            flags.add_defines(defines);
        }

        // always include libnx includes
        // TODO: maybe make this a build config option?
        flags.add_includes([env.libnx_include.display()]);

        let mut includes = Vec::with_capacity(build.includes.len() + 1);
        for dir in &build.includes {
            let path = env.root.join(dir).to_abs()?;
            includes.push(path.display().to_string());
        }

        flags.add_includes(includes);

        for source_dir in &build.sources {
            source_dirs.push(env.root.join(source_dir));
        }

        Ok(Self {
            env,
            flags,
            source_dirs,
            lib_objects: Vec::new(),
        })
    }

    pub fn check_lib_changed(
        &self,
        build: &Build,
        elf_mtime: Option<FileTime>,
    ) -> Result<bool, Error> {
        if build.libmegaton {
            let libmegaton_path = self
                .env
                .megaton_home
                .join("lib")
                .into_joined("build")
                .into_joined("bin")
                .into_joined("libmegaton.a");

            if !libmegaton_path.exists() {
                errorln!("Failed", "libmegaton is not built!");
                hintln!("Consider", "Build it with `megaton build --lib`");
                return Err(report!(Error::Dependency));
            }

            match system::get_mtime(libmegaton_path) {
                Ok(mtime) => {
                    if !system::up_to_date(mtime, elf_mtime) {
                        verboseln!("relinking because libmegaton changed");
                        return Ok(true);
                    }
                }
                Err(e) => {
                    verboseln!("failed to get mtime of libmegaton: {}", e);
                    return Ok(true);
                }
            }
        }

        // check if any other lib changed
        for libpath in &build.libpaths {
            for lib in &build.libraries {
                let lib_path = self.env.root.join(libpath).join(format!("lib{}.a", lib));
                if lib_path.exists() {
                    match system::get_mtime(lib_path) {
                        Ok(mtime) => {
                            if !system::up_to_date(mtime, elf_mtime) {
                                verboseln!("relinking because lib{} changed", lib);
                                return Ok(true);
                            }
                        }
                        Err(e) => {
                            verboseln!("failed to get mtime of libpath: {}", e);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    pub fn configure_linker(&mut self, build: &Build) -> Result<(), Error> {
        self.flags.set_init(build.entry_point());
        self.flags.set_version_script(self.env.verfile.display());

        if build.libmegaton {
            let lib_path = self.env.megaton_home.join("lib");
            let libmegaton_path = lib_path.join("build").into_joined("bin");

            let libmegaton = libmegaton_path.into_joined("libmegaton.a");
            if !libmegaton.exists() {
                errorln!("Failed", "libmegaton is not built!");
                hintln!("Consider", "Build it with `megaton build --lib`");
                return Err(report!(Error::Dependency));
            }

            self.lib_objects.push(libmegaton.display().to_string());

            let runtime_source = lib_path.into_joined("runtime");

            self.flags
                .add_ldscripts([runtime_source.into_joined("link.ld").display()]);
        }

        self.flags.add_libpaths(
            build
                .libpaths
                .iter()
                .map(|libpath| self.env.root.join(libpath))
                .collect::<Vec<_>>()
                .iter()
                .map(|x| x.display()),
        );
        self.flags.add_libraries(&build.libraries);
        self.flags.add_ldscripts(
            build
                .ldscripts
                .iter()
                .map(|ldscript| self.env.root.join(ldscript))
                .collect::<Vec<_>>()
                .iter()
                .map(|x| x.display()),
        );

        Ok(())
    }

    fn create_compdb_entry(&self, source_file: SourceFile, d_file: String) -> CompileDBEntry {
        match source_file.typ {
            SourceType::C => CompileDBEntry::new(
                &self.env.cc,
                &source_file.path,
                d_file,
                self.flags.cflags.iter().cloned(),
            ),
            SourceType::Cpp => CompileDBEntry::new(
                &self.env.cxx,
                &source_file.path,
                d_file,
                self.flags.cxxflags.iter().cloned(),
            ),
            SourceType::S => CompileDBEntry::new(
                &self.env.cxx,
                &source_file.path,
                d_file,
                ["-x".to_string(), "assembler-with-cpp".to_string()]
                    .into_iter()
                    .chain(self.flags.sflags.iter().cloned()),
            ),
        }
    }

    pub fn process_source(
        &self,
        source_path: &Path,
        check_compdb: bool,
        compdb: &CompileDB,
    ) -> Result<SourceResult, system::Error> {
        let source_file = match SourceFile::from_path(source_path)? {
            Some(x) => x,
            None => {
                return Ok(SourceResult::NotSource);
            }
        };

        let o_path = self
            .env
            .target_o
            .join(format!("{}.o", source_file.name_hash));
        let o_file = o_path.to_utf8()?;
        let d_path = self
            .env
            .target_o
            .join(format!("{}.d", source_file.name_hash));
        let d_file = d_path.to_utf8()?;
        if !o_path.exists() {
            // output doesn't exist
            verboseln!("will compile '{}' (output doesn't exist)", source_file.path);
            let cc = self.create_compdb_entry(source_file, d_file);
            return Ok(SourceResult::NeedCompile(cc, o_file));
        }
        // d file changed? (source included in d_file)
        if !are_deps_up_to_date(&d_path, &o_path)? {
            verboseln!("will compile '{}' (deps outdated)", source_file.path);
            let cc = self.create_compdb_entry(source_file, d_file);
            return Ok(SourceResult::NeedCompile(cc, o_file));
        }
        // dependencies didn't change.
        // Only rebuild if compile command changed
        if !check_compdb {
            return Ok(SourceResult::UpToDate(o_file));
        }
        let cc = self.create_compdb_entry(source_file, d_file);
        match compdb.get(&o_file) {
            Some(old_cc) => {
                if old_cc == &cc {
                    Ok(SourceResult::UpToDate(o_file))
                } else {
                    verboseln!("will compile '{}' (command changed)", cc.file);
                    Ok(SourceResult::NeedCompile(cc, o_file))
                }
            }
            None => {
                // no previous command found, (never built), need build
                verboseln!("will compile '{}' (never built)", cc.file);
                Ok(SourceResult::NeedCompile(cc, o_file))
            }
        }
    }

    pub fn link_start(&self, objects: &[String]) -> Result<Spawned, system::Error> {
        // use CXX for linking
        Command::new(&self.env.cxx)
            .args(
                self.flags
                    .ldflags
                    .iter()
                    .chain(objects.iter())
                    .chain(self.lib_objects.iter())
                    .chain(["-o".to_string(), self.env.elf.display().to_string()].iter()),
            )
            .silence_stdout()
            .pipe_stderr()
            .spawn()
    }
}

pub enum SourceResult {
    NotSource,
    UpToDate(String),
    NeedCompile(CompileDBEntry, String),
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

    let mut file_name_buf = String::new();
    let lines = file.lines();
    // skip the <target>: \ line
    for line in lines.skip(1) {
        let line = match line {
            Ok(x) => x,
            Err(_) => return Ok(false),
        };
        let part = line.trim().trim_end_matches('\\').trim_end();
        if part.ends_with(':') {
            break;
        }
        // 1. there could be multiple files per line
        // 2. there could be spaces in the file name, escaped by \
        for file_part in part.split(' ') {
            // handle escape
            if let Some(file_part) = file_part.strip_suffix('\\') {
                // copy is unavoidable to undo the escape
                // don't put spaces in your file names!
                file_name_buf.push_str(file_part);
                file_name_buf.push(' ');
                continue;
            }
            let d_mtime = if file_name_buf.is_empty() {
                system::get_mtime(file_part)?
            } else {
                file_name_buf.push_str(file_part);
                let t = system::get_mtime(&file_name_buf)?;
                file_name_buf.clear();
                t
            };
            if !system::up_to_date(d_mtime, o_mtime) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
