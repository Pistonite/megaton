// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Megaton contributors

// This module manages linking the mod and library
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

// use cu::{PathExtension, Spawn, fs::{self, DirEntry}, lv::P};
use cu::{info, pre::*};

use crate::{cmds::cmd_build::{
    compile::CompileDB,
    config::{Build, Config, Flags, Module},
}, env::environment};


pub fn get_ldscripts(build_config: &Build) -> String {
    build_config
        .ldscripts
        .iter()
        .map(|script| "-T ".to_owned() + script)
        .collect()
}

pub fn build_library_args(libraries: &Vec<String>) -> String {
    // e.g. given ["lib1.ld", "lib2.ld"], returns: "-L lib1.ld -L lib2.ld"
    libraries.iter().map(|l| "-l ".to_owned() + l).collect()
}

pub fn get_last_linker_command(module: &Module, profile: &str) -> Option<String> {
    let ld_cache = &module
        .target
        .join(profile)
        .join(module.name.clone())
        .join("ld.cache");
    cu::fs::read_string(ld_cache).ok()
}

pub fn get_output_path(module: &Module, profile: &str) -> PathBuf {
    module
        .target
        .join(profile)
        .join(module.name.clone())
        .canonicalize()
        .unwrap_or_else(|err| panic!("Failed to get output path! Error: {:?}", err))
}

pub fn get_cached_obj_files(module: &Module, profile: &str) -> Vec<PathBuf> {
    let mut obj_cache = &module
        .target
        .join(profile)
        .join(module.name.clone())
        .join("objects.cache");
    let file = cu::fs::read_string(obj_cache).expect("Failed to read object cache");
    file.lines().map(|l| PathBuf::from(l)).collect()
}

pub fn get_obj_files(module: &Module) -> Vec<PathBuf> {
    // scan target for obj files
    let target = &module.target;
    get_obj_files_in(target)
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
                result.push(path.path());
            }
        }
    }
    result
}

fn to_space_separated_str(paths: Vec<PathBuf>) -> String {
    paths
        .iter()
        .map(|p| p.to_str().unwrap().to_owned())
        .collect::<Vec<String>>()
        .join(" ")
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


pub fn needs_relink(
    compiler_did_something: bool,
    elf_path: PathBuf,
    compdb: &mut CompileDB,
    build_config: &Build,
    module: &Module,
    profile: &str,
) -> cu::Result<bool> {

    Ok(true)

    // if compiler_did_something {
    //     return cu::Result::Ok(true);
    // }

    // let output_path = get_output_path(module, profile)
    //     .to_str()
    //     .expect("Failed to get absolute output path!")
    //     .to_owned();
    // let obj_files = to_space_separated_str(get_obj_files(module));

    // let libraries = build_library_args(&build_config.ldscripts);
    // let old_command = get_last_linker_command(module, profile);
    // let ldscripts = get_ldscripts(build_config);
    // let new_ld_command = format!(
    //     "ld -o {} {} {} {}",
    //     output_path, ldscripts, obj_files, libraries
    // );
    // if let Some(old_command) = old_command
    //     && &new_ld_command == &old_command
    // {
    //     // ld command would be the same as before, nothing changed.
    //     cu::Result::Ok(false)
    // } else {
    //     compdb.ld_command = new_ld_command;
    //     cu::Result::Ok(true)
    // }
}


// pub async fn relink(
//     elf_path: PathBuf,
//     compdb: &mut CompileDB,
//     module: &Module,
// ) -> cu::Result<bool> {
//     let output_path = "";
//     let obj_files: Vec<PathBuf> = get_obj_files(module); // scan target folder (BuildConfig)
//     let libraries: Vec<&str> = vec!["-L"]; // BuildConfig
//     let ld = cu::which("ld")?;
//     let mut args = vec![];

//     args.push("-o");
//     args.push(output_path);
//     args.extend(obj_files.iter().map(|p| p.to_str().unwrap()));
//     args.push("-L");
//     args.extend(libraries);
//     let linker_command = ld
//         .command()
//         .args(args)
//         .stdin_null()
//         .stdoe(cu::pio::spinner("linking").info())
//         .co_spawn()
//         .await?
//         .0;
//     Ok(true)
// }

pub fn relink_sync(
    module_path: &PathBuf,
    compdb: &mut CompileDB,
    module: &Module,
    flags: &Flags,
) -> cu::Result<bool> {
    let elf_name = format!("{}.elf",module.name);
    let elf_path = module_path.join(elf_name);
    
    let obj_files: Vec<String> = get_obj_files(module)
        .iter().map(|o| o.display().to_string()).collect(); // scan target folder (BuildConfig)
    let libraries: Vec<String> = vec![]; // BuildConfig
    let env = environment();

    let command = link_start(env.cxx_path(), flags, obj_files, elf_path)?;
    command.wait_nz()?;
    info!("Linker finished!");
    Ok(true)
}

fn link_start(cxx: &Path, flags: &Flags, objects: Vec<String>, elf_path: PathBuf) -> cu::Result<cu::Child> {
    let output_arg = ["-o".to_string(), elf_path.display().to_string()];
    let args = flags.ldflags
                .iter()
                .chain(objects.iter())
                // .chain(self.lib_objects.iter())
                .chain(output_arg.iter());
    info!("{:?}", &args);
    let res = cxx.command()
        .args(
            args,
        )
        .stdin_null()
        .stdoe(cu::pio::spinner("linking").info())
        .spawn()?
        .0;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_obj_files() {
        let mut path = PathBuf::from("test/test_get_obj_files");
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
        let mut path = PathBuf::from("test/test_get_obj_files/empty");
        let result = get_obj_files_in(&path);
        assert_eq!(result.len(), 0, "{:#?}", result);
    }

    #[test]
    fn test_to_space_separated_str() {
        let input = vec![
            PathBuf::from("/a/b/c/def.txt"),
            PathBuf::from("test/example.o"),
        ];
        assert_eq!(
            to_space_separated_str(input),
            "/a/b/c/def.txt test/example.o"
        )
    }

    #[test]
    fn test_to_space_separated_str_empty_array() {
        let input = vec![];
        assert_eq!(to_space_separated_str(input), "")
    }
}
