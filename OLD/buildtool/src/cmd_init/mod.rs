use buildcommon::prelude::*;
use std::path::Path;

use crate::error::Error;

macro_rules! init_file {
    ($file:literal, $target:ident) => {{
        let content = include_str!(concat!("template/", $file));
        let target_path = $target.join($file);
        system::write_file(&target_path, content).change_context(Error::InitFile($file))
    }};
}

pub fn run(path: &str) -> Result<(), Error> {
    let path = Path::new(path);
    if path.exists() {
        if path.is_file() {
            errorln!("Failed", "Path is a file: {}", path.display());
            bail!(Error::AlreadyExists);
        }
        let mut dir = std::fs::read_dir(path).change_context(Error::AlreadyExists)?;
        if dir.next().is_some() {
            errorln!("Failed", "Path is not empty: {}", path.display());
            bail!(Error::AlreadyExists);
        }
    } else {
        system::ensure_directory(path).change_context(Error::InitDir)?;
        infoln!("Created", "{}", path.display());
    }

    let path = path.to_abs().change_context(Error::InitDir)?;

    // Megaton.toml
    let module_name = path
        .file_name()
        .ok_or(Error::InitDir)
        .attach_printable("cannot init in root dir")?
        .to_string_lossy();
    let module_name_escaped = module_name
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace(' ', "_");

    let megaton_path = path.join("Megaton.toml");
    let megaton_toml = include_str!("template/Megaton.toml");
    let megaton_toml = megaton_toml.replacen("NAMEPLACEHOLDER", &module_name_escaped, 1);
    system::write_file(&megaton_path, megaton_toml)
        .change_context(Error::InitFile("Megaton.toml"))?;

    system::ensure_directory(path.join("src")).change_context(Error::InitDir)?;

    init_file!("src/main.cpp", path)?;
    init_file!(".clang-format", path)?;
    init_file!(".clangd", path)?;
    init_file!(".gitignore", path)?;

    infoln!("Initialized", "empty project `{}`", module_name);

    Ok(())
}
