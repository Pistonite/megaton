use std::path::{Path, PathBuf};

use cu::json;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cmds::cmd_build::compile::{compile_command::{CompileCommand, devkitpro_includes}};



#[derive(Serialize, Deserialize)]
pub struct CompileRecord {
    pub args: Vec<String>,
    pub source: PathBuf,
    pub pathhash: usize,
}

impl From<&CompileCommand> for CompileRecord {
    fn from(value: &CompileCommand) -> Self {
        Self {
            args: value.args.clone(),
            source: value.source.clone(),
            pathhash: value.pathhash
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct CompileDB {
    commands: Vec<CompileRecord>, // ordered by completion time
}

impl CompileDB {
    pub fn new(compilation_results: Vec<cu::Result<CompileCommand>>) -> Self {
        let successful_compilations = compilation_results.iter()
            .filter_map(|result| result.as_ref().map(|r| CompileRecord::from(r)).ok())
            .collect::<Vec<_>>();
        Self { commands: successful_compilations }
    }

    // Creates a new compile record and adds it to the db
    pub fn update(&mut self, command: &CompileCommand) {
        self.commands.push(CompileRecord::from(command));
    }

    pub fn update_all(&mut self, compilation_results: Vec<cu::Result<CompileCommand>>){
        compilation_results.iter().for_each(|result| {
            if let Ok(compile_command) = result {
                self.update(compile_command);
            }
        })
    }

    pub fn save(&self, path: &PathBuf) -> cu::Result<()> {
        let file = std::fs::File::create(path)?;
        json::write(file, self)
    }

    pub fn find_record(&self, source_pathhash: usize) -> Option<&CompileRecord> {
        self.commands.iter().find(|cmd| cmd.pathhash == source_pathhash)
    }

    /*
    Saves compilation results to compile_commands.json for use with clangd
    For more details, see: https://clang.llvm.org/docs/JSONCompilationDatabase.html 
     */
    pub fn save_cc_json(&self, path: &Path) -> cu::Result<()> {
        let file = std::fs::File::create(path)?;
        let entries = self
            .commands
            .iter()
            .map(|cc| CCJsonEntry::from(cc))
            .collect::<Vec<CCJsonEntry>>();

        json::write_pretty(file, &entries)
    }

    // pub fn save_command_log(&self, path: &PathBuf) -> cu::Result<()> {
    //     let mut file = std::fs::File::create(path)?;
    //     let content = self
    //         .commands
    //         .iter().map(|cc| cc.command())
    //         .collect::<Vec<_>>()
    //         .join("\n");
    //     let content = content + "\n";
    //     file.write_all(content.as_bytes())?;
    //     Ok(())
    // }
}


#[derive(Serialize)]
struct CCJsonEntry {
    arguments: Vec<String>,
    directory: String,
    file: String,
}

impl From<&CompileRecord> for CCJsonEntry {
    fn from(value: &CompileRecord) -> Self {
        let mut arguments = value
            .args.clone()
            .into_iter()
            .filter(|x| {
                let re = Regex::new(r"-mtune=.+|-march=.+|-mtp=.+").unwrap();
                !re.is_match(x)
            })
            .collect::<Vec<_>>();

        arguments.extend(
            devkitpro_includes()
                .into_iter()
                .map(|i| format!("-isystem {i}"))
                .collect::<Vec<_>>(),
        );

        let directory = PathBuf::from(".")
            .canonicalize()
            .unwrap()
            .display()
            .to_string();
        let file = value.source.display().to_string();
        Self {
            arguments,
            directory,
            file,
        }
    }
}

