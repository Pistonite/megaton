// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 Megaton contributors

// This modules handles compiling c/c++/asm/rust code
use std::{
    io::Write,
    path::{Path, PathBuf},
};

use cu::pre::*;
use regex::Regex;

use crate::{env::environment};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct LinkCommand {
    linker: PathBuf,
    args: Vec<String>,
}

impl LinkCommand {
    pub fn new(ld_path: &Path, args: &[String]) -> Self {
        Self {
            linker: ld_path.to_path_buf(),
            args: args.to_vec(),
        }
    }

    fn command(&self) -> String {
        format!("{} {}", self.linker.display(), self.args.join(" "))
    }
}

fn get_obj_files_in(target: &PathBuf) -> Vec<PathBuf> {
    let paths = cu::fs::read_dir(target).unwrap();
    let mut result = Vec::new();
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

pub struct LinkContext {
    module_obj_path: PathBuf,
    output_elf_path: PathBuf,
    verfile_path: PathBuf,
    lib_linkldscript_path: PathBuf,

    entrypoint: Option<String>,
    old_link_command: Option<String>,
    libraries: Vec<String>,
    ldscripts: Vec<String>,
    libpaths: Vec<String>,
    flags: Vec<String>,
    compilation_occurred: bool,
}

pub async fn relink(
    lc: LinkContext
) -> cu::Result<bool> {
    // Returns: whether linking was performed
    let env = environment();

    let mod_objs: Vec<String> = get_obj_files_in(&lc.module_obj_path)
        .iter()
        .map(|o| o.display().to_string())
        .collect(); // scan target folder (BuildConfig)

    let output_arg = [
        "-o".to_string(),
        lc.output_elf_path.display().to_string(),
    ];

    let mut args = lc.flags.clone();
    let mut ldscripts = lc.ldscripts.clone();

    let main_ldscript_path = lc
        .lib_linkldscript_path
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    ldscripts.insert(0, main_ldscript_path);

    let linker_args: Vec<Vec<String>> = [&lc.libraries, &lc.libpaths, &ldscripts].iter().map(|paths| {
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

    let entrypoint = lc.entrypoint
        .clone()
        .unwrap_or("__megaton_module_entry".to_owned());
    args.push(format!("-Wl,-init={}", entrypoint));
    let verfile_path = &lc.verfile_path;
    create_verfile(verfile_path, &entrypoint)?;
    args.push(format!("-Wl,--version-script={}", verfile_path.display()));

    args.extend(ldscripts);
    args.extend(libpaths);
    args.extend(libraries);
    args.extend(mod_objs);
    args.extend(output_arg);
    let cxx = env.cxx_path();

    let link_cmd = LinkCommand::new(cxx, &args);
    if !lc.compilation_occurred
        && let Some(old_link_cmd) = lc.old_link_command
        && *old_link_cmd == link_cmd.command()
    {
        return Ok(false); // skip compilation
    }

    let command = cxx
        .command()
        .args(args)
        .stdin_null()
        .stdoe(cu::pio::spinner("linking").info())
        .co_spawn().await?
        .0;

    cu::debug!("Link command: {}", link_cmd.command());
    command.co_wait_nz().await?;
    cu::info!("Linker finished!");
    Ok(true)
}

pub async fn build_nso(elf_path: &PathBuf, nso_path: &PathBuf) -> cu::Result<()> {
    let elf2nso = environment().elf2nso_path();

    let command = elf2nso
        .command()
        .args([elf_path, nso_path])
        .stdin_null()
        .stdoe(cu::pio::spinner("Building NSO").info())
        
        .co_spawn().await
        .context("Failed to build NSO!")?
        .0;

    let result = command.co_wait_nz().await;
    match result.is_err() {
        true => Ok(()),
        false => Err(cu::Error::msg(format!(
            "{:#?} failed to execute:{:#?}", elf2nso, result)
        ))
    }
}


fn create_verfile(verfile: &PathBuf, entry: &str) -> cu::Result<()> {
    cu::debug!("creating verfile");
    let verfile_before = "{\n\tglobal:\n\n";
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
