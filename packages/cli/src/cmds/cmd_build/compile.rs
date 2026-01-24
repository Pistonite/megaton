// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

// This modules handles compiling c/c++/asm/rust code
use std::{
    io::Write,
    path::{Path, PathBuf},
};

use cu::pre::*;

use super::Flags;
use crate::{
    cmds::cmd_build::{
        BTArtifacts,
        config::{Build, Config},
    },
    env::environment,
};

#[derive(Serialize, Deserialize, Default)]
pub struct CompileDB {
    commands: Vec<CompileRecord>, // ordered by completion time
    cc_version: String,
    cxx_version: String,
}

impl CompileDB {
    // Creates a new compile record and adds it to the db
    fn update(&mut self, command: CompileRecord) {
        self.commands.push(command);
    }

    pub fn save(&self, path: &PathBuf) -> cu::Result<()> {
        let file = std::fs::File::create(path)?;
        json::write(file, self)
    }

    pub fn save_command_log(&self, path: &PathBuf) -> cu::Result<()> {
        let mut file = std::fs::File::create(path)?;
        let content = self
            .commands
            .iter()
            .map(|record| match record {
                CompileRecord::Compile(compile_command) => compile_command.command(),
                CompileRecord::Link(link_command) => link_command.command(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        let content = content + "\n";
        file.write(content.as_bytes())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
enum CompileRecord {
    Compile(CompileCommand),
    Link(LinkCommand),
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct LinkCommand {
    linker: PathBuf,
    args: Vec<String>,
}
impl LinkCommand {
    pub fn new(ld_path: &Path, args: &Vec<String>) -> Self {
        Self {
            linker: ld_path.to_path_buf(),
            args: args.to_vec(),
        }
    }

    fn command(&self) -> String {
        format!("{} {}", self.linker.display(), self.args.join(" "))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct CompileCommand {
    pathhash: usize,
    compiler: PathBuf,
    source: PathBuf,
    args: Vec<String>,
}

impl CompileCommand {
    fn new(
        compiler_path: &Path,
        src_file: &Path,
        out_file: &Path,
        dep_file: &Path,
        flags: &Vec<String>,
        includes: Vec<String>,
    ) -> cu::Result<Self> {
        let mut argv = flags.clone();
        argv.push("-MMD".to_owned());
        argv.push("-MP".to_owned());
        argv.push("-MF".to_owned());
        argv.push(dep_file.as_utf8()?.to_string());

        let mut includes = includes;
        includes.extend(devkitpro_includes(compiler_path)?);
        let includes = includes
            .iter()
            .filter_map(|i| {
                let path = PathBuf::from(i);
                path.canonicalize()
                    .inspect_err(|e| cu::error!("cant find include {} {}", path.display(), e))
                    .ok()
            })
            .map(|i| format!("-I{}", i.as_os_str().to_str().unwrap()))
            .collect::<Vec<String>>();
        argv.extend(includes);

        argv.push(String::from("-c"));

        argv.push(format!("-o{}", out_file.as_utf8()?));

        argv.push(src_file.as_utf8()?.to_string());

        cu::trace!(
            "Compiler command: \n{} {} {}",
            &compiler_path.display(),
            &src_file.display(),
            &argv.join(" ")
        );

        let src_path = src_file.to_path_buf();
        Ok(Self {
            pathhash: fxhash::hash(&src_path),
            compiler: compiler_path.to_path_buf(),
            source: src_path,
            args: argv,
        })
    }

    fn execute(&self) -> cu::Result<()> {
        cu::trace!(
            "Executing CompileCommand: \n{} {}",
            &self.compiler.display(),
            &self.args.join(" ")
        );
        let child = cu::CommandBuilder::new(&self.compiler.as_os_str())
            .args(self.args.clone())
            .stdoe(cu::pio::inherit()) // todo: log to file
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        Ok(())
    }

    fn command(&self) -> String {
        format!("{} {}", self.compiler.display(), self.args.join(" "))
    }
}

fn devkitpro_includes(compiler_path: &Path) -> cu::Result<Vec<String>> {
    let devkitpaths = [
        "/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/?ver?".to_owned(),
        "/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/?ver?/aarch64-none-elf".to_owned(),
        "/opt/devkitpro/devkitA64/aarch64-none-elf/include/c++/?ver?/backward".to_owned(),
        "/opt/devkitpro/devkitA64/lib/gcc/aarch64-none-elf/?ver?/include".to_owned(),
        "/opt/devkitpro/devkitA64/lib/gcc/aarch64-none-elf/?ver?/include-fixed".to_owned(),
        "/opt/devkitpro/devkitA64/aarch64-none-elf/include".to_owned(),
    ];
    let (gcc, mut output) = cu::CommandBuilder::new(compiler_path.as_os_str())
        .arg("--version")
        .stdout(cu::pio::lines()) // todo: log to file
        .stderr_null()
        .stdin_null()
        .spawn()?;
    let verline = output.next().unwrap()?;
    let verstring = verline.split(" ").nth(2).unwrap();
    let new_includes: Vec<String> = devkitpaths
        .iter()
        .map(|path| path.replace("?ver?", verstring))
        .collect();
    Ok(new_includes)
    // gcc.wait_nz()?;
}

#[derive(Serialize)]
struct CompileCommandsClangd {
    commands: Vec<CommandObjectClangd>,
}

#[derive(Serialize)]
struct CommandObjectClangd {
    arguments: Vec<String>,
    directory: String,
    file: String,
}

impl CommandObjectClangd {
    fn new(arguments: Vec<String>, directory: String, file: String) -> Self {
        Self {
            arguments,
            directory,
            file,
        }
    }
}

// impl From<CompileCommand> for CommandObjectClangd {
//     fn from(value: CompileCommand) -> Self {
//         Self {}
//     }
// }

// A source file and its corresponding artifacts
pub struct SourceFile {
    lang: Lang,
    path: PathBuf,
    pathhash: usize,
    basename: String,
}

impl SourceFile {
    pub fn new(lang: Lang, path: PathBuf) -> cu::Result<Self> {
        let basename = cu::PathExtension::file_name_str(&path)
            .context("path is not utf-8")?
            .to_owned();
        let pathhash = fxhash::hash(&path);
        Ok(Self {
            lang,
            path,
            pathhash,
            basename,
        })
    }

    pub fn compile(
        &self,
        flags: &Flags,
        includes: Vec<String>,
        compile_db: &mut CompileDB,
        output_path: &PathBuf,
    ) -> cu::Result<bool> {
        if !output_path.exists() {
            cu::fs::make_dir(&output_path).unwrap();
            cu::info!(
                "Output path {:?} exists={}",
                &output_path,
                &output_path.exists()
            );
        }
        let o_path = output_path.join(format!("{}-{}.o", self.basename, self.pathhash));
        let d_path = output_path.join(format!("{}-{}.d", self.basename, self.pathhash));

        let (comp_path, comp_flags) = match self.lang {
            Lang::C => (environment().cc_path(), &flags.cflags),
            Lang::Cpp => (environment().cxx_path(), &flags.cxxflags),
            Lang::S => (environment().cc_path(), &flags.sflags),
        };

        let comp_command = CompileCommand::new(
            comp_path,
            &self.path,
            &o_path,
            &d_path,
            &comp_flags,
            includes,
        )?;

        compile_db.update(CompileRecord::Compile(comp_command.clone()));

        if self.need_recompile(compile_db, &o_path, &d_path, &comp_command)? {
            // Ensure source and artifacts have the same timestamp
            let src_time = cu::fs::get_mtime(&self.path)?.unwrap();

            // Compile and update record
            comp_command.execute()?;

            cu::fs::set_mtime(o_path, src_time)?;
            if d_path.exists() {
                cu::fs::set_mtime(d_path, src_time)?;
            }
            Ok(true)
            // compile_db.update(comp_command)?;
        } else {
            Ok(false)
        }
    }

    fn need_recompile(
        &self,
        compile_db: &CompileDB,
        o_path: &Path,
        d_path: &Path,
        command: &CompileCommand,
    ) -> cu::Result<bool> {
        // Check if record exists
        let comp_record = compile_db.commands.iter().find(|command| match command {
            CompileRecord::Compile(compile_command) => compile_command.pathhash == self.pathhash,
            CompileRecord::Link(_) => false,
        });

        let comp_record = match comp_record {
            Some(CompileRecord::Compile(cmd)) => cmd,
            _ => return Ok(true),
        };

        // Check if artifacts exist
        if !o_path.exists() || !d_path.exists() {
            return Ok(true);
        }

        // Check if artifacts are up to date
        if cu::fs::get_mtime(o_path)? != cu::fs::get_mtime(&self.path)?
            || cu::fs::get_mtime(d_path)? != cu::fs::get_mtime(&self.path)?
        {
            return Ok(true);
        }

        let d_file_contents = cu::fs::read_string(d_path)?;
        let depfile = match depfile::parse(&&d_file_contents) {
            Ok(depfile) => depfile,

            // Make sure our errors are all cu compatible
            Err(_) => return Err(cu::Error::msg("Failed to parse depfile")),
        };

        for dep in depfile.recurse_deps(o_path.as_utf8()?) {
            if cu::fs::get_mtime(PathBuf::from(dep))? != cu::fs::get_mtime(&self.path)? {
                return Ok(true);
            }
        }

        if command != comp_record {
            return Ok(true);
        }

        // if (self.lang == Lang::Cpp && compile_db.cxx_version != env.cxx_version)
        //     || compile_db.cc_version != env.cc_version
        // {
        //     return Ok(true);
        // }

        // No need to recompile!
        Ok(false)
    }
}

// Specifies source language (rust is managed separately)
#[derive(PartialEq, Eq)]
pub enum Lang {
    C,
    Cpp,
    S,
}

fn get_obj_files_in(target: &PathBuf) -> Vec<PathBuf> {
    let paths = cu::fs::read_dir(target).unwrap();
    let mut result: Vec<PathBuf> = vec![];

    for path in paths {
        if let Ok(path) = path
            && let Ok(ft) = path.file_type()
        {
            if ft.is_dir() {
                result.extend(get_obj_files_in(&path.path()));
            } else if ft.is_file() && is_file_obj(&path) {
                result.push(path.path().canonicalize().unwrap());
            }
        }
    }
    result
}

fn is_file_obj(path: &cu::fs::DirEntry) -> bool {
    let name = path.file_name();
    let name = name.to_str();
    if let Some(name) = name {
        name.ends_with(".o")
    } else {
        false
    }
}

pub fn relink(
    bt_artifacts: &BTArtifacts,
    compile_db: &mut CompileDB,
    config: &Config,
    flags: &Flags,
    build: &Build,
    rust_staticlib: Option<PathBuf>,
    compilation_occurred: bool,
) -> cu::Result<bool> {
    // Returns: whether linking was performed
    let env = environment();

    let mod_objs: Vec<String> = get_obj_files_in(&bt_artifacts.module_root)
        .iter()
        .map(|o| o.display().to_string())
        .collect(); // scan target folder (BuildConfig)
    // TODO: Unpack lib
    let lib_objs: Vec<String> = get_obj_files_in(&bt_artifacts.lib_obj)
        .iter()
        .map(|o| o.display().to_string())
        .collect();

    let output_arg = [
        "-o".to_string(),
        bt_artifacts.elf_path.display().to_string(),
    ];

    let mut args = flags.ldflags.clone();
    let mut ldscripts = build.ldscripts.clone();

    let main_ldscript_path = bt_artifacts
        .lib_linkldscript
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    ldscripts.insert(0, main_ldscript_path);

    let linker_args: Vec<Vec<String>> = vec![&build.libraries, &build.libpaths, &ldscripts].iter().map(|paths| {
        paths.iter().filter_map(|path| {
            match PathBuf::from(path).canonicalize() {
                Ok(abs_path) => Some(abs_path.display().to_string()),
                Err(e) => {
                    cu::error!("Error when building link flags. Failed to canonicalize path to {}. Error: {}", path, e);
                    None
                },
            }
        }).collect()
    }).collect();

    let libraries = linker_args[0].clone();
    let libpaths: Vec<String> = linker_args[1]
        .iter()
        .map(|lp| format!("-L{}", lp))
        .collect();
    let ldscripts: Vec<String> = linker_args[2]
        .iter()
        .map(|lp| format!("-Wl,-T,{}", lp))
        .collect();

    let entrypoint = &config
        .megaton
        .entry
        .clone()
        .unwrap_or("__megaton_module_entry".to_owned());
    args.push(format!("-Wl,-init={}", entrypoint));
    let verfile_path = &bt_artifacts.verfile_path;
    create_verfile(verfile_path, entrypoint)?;
    args.push(format!(
        "-Wl,--version-script={}",
        verfile_path.display().to_string()
    ));

    args.extend(ldscripts);
    args.extend(libpaths);
    args.extend(libraries);
    args.extend(mod_objs);
    if let Some(rust_staticlib_path) = rust_staticlib {
        args.push(rust_staticlib_path.display().to_string());
    }
    args.extend(lib_objs);
    args.extend(output_arg);
    let cxx = env.cxx_path();

    let link_cmd = LinkCommand::new(cxx, &args);
    let old_link_cmd: Option<&LinkCommand> =
        compile_db
            .commands
            .iter()
            .find_map(|cmd| -> Option<&LinkCommand> {
                match cmd {
                    CompileRecord::Compile(_) => None,
                    CompileRecord::Link(link_command) => Some(link_command),
                }
            });
    if !compilation_occurred
        && let Some(old_link_cmd) = old_link_cmd
        && *old_link_cmd == link_cmd
    {
        return Ok(false); // skip compilation
    }

    compile_db.update(CompileRecord::Link(link_cmd.clone()));
    let command = cxx
        .command()
        .args(args)
        .stdin_null()
        .stdoe(cu::pio::spinner("linking").info())
        .spawn()?
        .0;

    command.wait_nz()?;
    cu::debug!("Link command: {}", link_cmd.command());
    cu::info!("Linker finished!");
    Ok(true)
}

pub fn build_nso(elf_path: &PathBuf, nso_path: &PathBuf) -> cu::Result<()> {
    let elf2nso = environment().elf2nso_path();

    let command = elf2nso
        .command()
        .args([elf_path, nso_path])
        .stdin_null()
        .stdoe(cu::pio::spinner("Building NSO").info())
        .spawn()
        .context("Failed to build NSO!")?
        .0;

    let result = command.wait()?;
    match result.success() {
        true => Ok(()),
        false => Err(cu::Error::msg(format!(
            "Failed to build NSO with exit code {}",
            result.code().unwrap_or_else(|| {
                cu::error!("Failed to get exit code!");
                -1
            })
        ))),
    }
}

fn create_verfile(verfile: &PathBuf, entry: &str) -> cu::Result<()> {
    cu::debug!("creating verfile");
    let verfile_before = "{\n\tglobal:\n";
    let verfile_after = ";\n\tlocal: *;\n};";
    let verfile_data = format!("{}{}{}", verfile_before, entry, verfile_after);
    cu::fs::write(verfile, &verfile_data)?;
    cu::debug!("Created verfile");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_obj_files() {
        let path = PathBuf::from("test/test_get_obj_files");
        let result = get_obj_files_in(&path);
        assert!(
            result.contains(&PathBuf::from("test/test_get_obj_files/file1.o")),
            "{:#?}",
            result
        );
        assert!(
            result.contains(&PathBuf::from("test/test_get_obj_files/nested/file2.o")),
            "{:#?}",
            result
        );
    }

    #[test]
    fn test_get_obj_files_empty() {
        let path = PathBuf::from("test/test_get_obj_files/empty");
        let result = get_obj_files_in(&path);
        assert_eq!(result.len(), 0, "{:#?}", result);
    }
}
