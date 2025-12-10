

// exists: Path -> bool. Does there exist an ELF at Path
    // if elf exists at path, recreate/relink optionally
    // else: always remake elf
// make_nso: convert elf at Path to nso
// relink: take existing elf and relink using 
// need_relink: 
    // similar logic to needs_recompiled
    // Relink is needed if: 
        // If the ELF does not exist 
        // If any objects were re-compiled
            // passed in as arg
        // If the list of objects changed (new source files are added without changing any existing files, or some source files are deleted) 
            // passed in as arg
        // If any library changed, including the one cargo produced 
            // passed in as arg

        // If the linker command change 
            // read Config
        // If any linker scripts changed 
            // what? and how?
        
// check: read from Config and check for banned symbols - already implemented in old megaton

// args to ld: .exe name, list of .o and .a files


// "<version of ld> [.o files, .a files] [.ld files] [args]"

use std::{path::PathBuf, str::FromStr};

use cu::{PathExtension, Spawn, fs::{self, DirEntry}, lv::P};

use crate::cmds::cmd_build::{compile::CompileDB, config::Config};

pub async fn needs_relink(compiler_did_something: bool, elf_path: PathBuf, compdb: &mut CompileDB, config: &Config) -> cu::Result<bool> {
    if compiler_did_something  {
        return cu::Result::Ok(true);
    }

    // linker args:
        // output: -o
        // obj files  
        // libraries (-L [/path/to/libraries, ...])

    let output_path = 1;
    let obj_files = to_space_separated_str(get_obj_files(config));

    let libraries = 3;
    let old_command = &compdb.ld_command;
    let new_ld_command = format!("ld -o {} {} {}", output_path, obj_files, libraries);
    if &new_ld_command != old_command {
        compdb.ld_command = new_ld_command;
        cu::Result::Ok(true)
    } else {
        cu::Result::Ok(false)
    }    
}

pub fn get_obj_files(config: &Config) -> Vec<PathBuf> {
    // for current profile
        // for every module
            // target/profile/module/o - folder containing .o files
        
    let profile = config.profile.default.clone().unwrap_or("none".to_owned());
    let current_profile = config.build.get_profile(profile.as_str());
    let mut target: PathBuf = PathBuf::from_str(&config.module.target).unwrap(); // todo: scrutinize
    // scan target/o for obj files
    target.push("o");
    get_obj_files_in(target)
}

fn get_obj_files_in(target: PathBuf) -> Vec<PathBuf> {
    let paths = cu::fs::read_dir(target).unwrap();
    let mut result: Vec<PathBuf> = vec![];

    for path in paths {
        if let Ok(path) = path && let Ok(ft) = path.file_type() {
            if ft.is_dir() {
                result.extend(get_obj_files_in(path.path()));
            }
            else if ft.is_file() && is_file_obj(&path) {
                result.push(path.path());
            }
        }  
    }
    result
}

fn to_space_separated_str(paths: Vec<PathBuf>) -> String {
    paths.iter()
        .map(|p| 
            p.to_str().unwrap().to_owned())
        .collect::<Vec<String>>()
        .join(" ")
}

fn is_file_obj(path: &DirEntry) -> bool {
    let name = path.file_name();
    let name = name.to_str();
    if let Some(name) = name {
        name.ends_with(".o")
    } else {
        false
    }
}

pub async fn relink(elf_path: PathBuf, compdb: &mut CompileDB, config: &Config) -> cu::Result<bool> {
    let output_path = "";
    let obj_files: Vec<PathBuf> = get_obj_files(config);     // scan target folder (BuildConfig)
    let libraries: Vec<&str> = vec!["-L"]; // BuildConfig
    let ld = cu::which("ld")?;
    let mut args = vec![];

    args.push("-o");
    args.push(output_path);
    args.extend(obj_files.iter().map(|p| p.to_str().unwrap()));
    args.push("-L");
    args.extend(libraries);
    let linker_command = ld.command()
        .args(args)
        .stdin_null()
        .stdoe(cu::pio::spinner("linking").info())
        .co_spawn().await?.0;

    Ok(true)

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_obj_files() {
        let mut path = PathBuf::from("test");
        let result = get_obj_files_in(path);
        assert!(result.contains(&PathBuf::from("test/test_get_obj_files/file1.o")), "{:#?}", result);
        assert!(result.contains(&PathBuf::from("test/test_get_obj_files/nested/file2.o")), "{:#?}", result);
    }


    #[test]
    fn test_to_space_separated_str() {
        let input = vec![PathBuf::from("/a/b/c/def.txt"), PathBuf::from("test/example.o")];
        assert_eq!(to_space_separated_str(input), "/a/b/c/def.txt test/example.o")
    }
}