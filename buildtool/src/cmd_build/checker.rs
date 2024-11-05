use buildcommon::prelude::*;

use std::collections::BTreeSet;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::mpsc;

use buildcommon::env::ProjectEnv;
use buildcommon::system::{Command, Executor, Task};
use regex::Regex;

use super::config::Check;

use crate::error::Error;

error_context!(pub LoadChecker, |r| -> Error {
    errorln!("Failed", "Loading checker config");
    r.change_context(Error::CreateChecker)
});
pub fn load(
    env: &ProjectEnv,
    config: Check,
    executer: &Executor,
) -> ResultIn<Checker, LoadChecker> {
    let mut tasks = Vec::with_capacity(config.symbols.len());
    let (send, recv) = mpsc::channel();
    for path in &config.symbols {
        let path = env.root.join(path).to_abs()?;
        let file = system::buf_reader(&path)?;

        let id = env.from_root(&path).display().to_string();
        let send = send.clone();
        let task = executer.execute(move || {
            process_objdump_syms(&id, file.lines().map_while(std::result::Result::ok), send)?;
            Ok(())
        });
        tasks.push(task);
    }

    Ok(Checker {
        data: CheckData::new(env, config),
        tasks,
        recv: Some(recv),
    })
}

pub struct Checker {
    data: CheckData,
    tasks: Vec<Task<Result<(), Error>>>,
    recv: Option<mpsc::Receiver<String>>,
}

error_context!(pub ObjdumpSymbols, |r| -> Error {
    r.change_context(Error::ObjdumpSymbols)
});
error_context!(pub ProcessInstructions, |r| -> Error {
    r.change_context(Error::ProcessInstructions)
});

impl Checker {
    pub fn check_symbols(
        &mut self,
        executer: &Executor,
    ) -> ResultIn<CheckSymbolTask, ObjdumpSymbols> {
        // run objdump -T
        let mut child = Command::new(&self.data.objdump)
            .args(args!["-T", self.data.elf])
            .piped()
            .spawn()?;
        let elf_symbols = child
            .take_stdout()
            .ok_or(system::Error::Expect("failed to get output of objdump -T"))?;

        let (elf_send, elf_recv) = mpsc::channel();
        let dump_task = executer.execute(move || {
            process_objdump_syms(
                "(output of `objdump -T`)",
                elf_symbols.lines().map_while(std::result::Result::ok),
                elf_send,
            )
        });

        let ignore = std::mem::take(&mut self.data.config.ignore);
        let recv = self.recv.take().unwrap();
        let check_task = executer.execute(move || {
            let mut loaded_symbols = BTreeSet::new();
            while let Ok(symbol) = recv.recv() {
                loaded_symbols.insert(symbol);
            }
            let mut missing_symbols = vec![];
            while let Ok(symbol) = elf_recv.recv() {
                if ignore.contains(&symbol) {
                    continue;
                }
                if !loaded_symbols.contains(&symbol) {
                    missing_symbols.push(symbol);
                }
            }
            missing_symbols
        });
        let wait_task = executer.execute(move || -> ResultIn<(), ObjdumpSymbols> {
            let mut child = child.wait()?;
            let result = child.check();
            if result.is_err() {
                child.dump_stderr("Error");
            }
            result?;
            Ok(())
        });

        Ok(CheckSymbolTask {
            dump_task,
            check_task,
            wait_task,
            load_tasks: std::mem::take(&mut self.tasks),
        })
    }

    pub fn check_instructions(
        &self,
        executer: &Executor,
    ) -> ResultIn<CheckInstructionTask, ProcessInstructions> {
        let mut child = Command::new(&self.data.objdump)
            .args(args!["-d", self.data.elf])
            .piped()
            .spawn()?;
        let elf_instructions = child
            .take_stdout()
            .ok_or(system::Error::Expect("failed to get output of objdump -d"))?;
        let (elf_send, elf_recv) = mpsc::channel();
        let dump_task = executer.execute(move || {
            process_objdump_insts(
                elf_instructions.lines().map_while(std::result::Result::ok),
                elf_send,
            );
        });

        // These instructions will cause console to Instruction Abort
        // (potentially due to permission or unsupported instruction?)
        let mut disallowed_regexes = vec![
            Regex::new(r"^msr\s*spsel").unwrap(),
            Regex::new(r"^msr\s*daifset").unwrap(),
            Regex::new(r"^mrs\.*daif").unwrap(),
            Regex::new(r"^mrs\.*tpidr_el1").unwrap(),
            Regex::new(r"^msr\s*tpidr_el1").unwrap(),
            Regex::new(r"^hlt").unwrap(),
        ];
        let extra = &self.data.config.disallowed_instructions;
        if !extra.is_empty() {
            disallowed_regexes.reserve_exact(extra.len());
            for s in extra {
                disallowed_regexes.push(Regex::new(s)?);
            }
        }
        let check_task = executer.execute(move || {
            let mut output = vec![];
            while let Ok(inst) = elf_recv.recv() {
                for regex in &disallowed_regexes {
                    if regex.is_match(&inst.1) {
                        output.push(format!("{}: {}", inst.0, inst.1));
                        break;
                    }
                }
            }
            output
        });
        let wait_task = executer.execute(move || -> ResultIn<(), ProcessInstructions> {
            let mut child = child.wait()?;
            let result = child.check();
            if result.is_err() {
                child.dump_stderr("Error");
            }
            result?;
            Ok(())
        });

        Ok(CheckInstructionTask {
            dump_task,
            wait_task,
            check_task,
        })
    }
}

struct CheckData {
    objdump: PathBuf,
    elf: PathBuf,
    config: Check,
}

impl CheckData {
    pub fn new(env: &ProjectEnv, config: Check) -> Self {
        Self {
            objdump: env.objdump.clone(),
            elf: env.elf.clone(),
            config,
        }
    }
}

pub struct CheckSymbolTask {
    dump_task: Task<Result<(), Error>>,
    check_task: Task<Vec<String>>,
    wait_task: Task<ResultIn<(), ObjdumpSymbols>>,
    load_tasks: Vec<Task<Result<(), Error>>>,
}

impl CheckSymbolTask {
    pub fn wait(self) -> Result<Vec<String>, Error> {
        for task in self.load_tasks {
            task.wait()?;
        }
        self.dump_task.wait()?;
        self.wait_task.wait()?;
        let result = self.check_task.wait();
        Ok(result)
    }
}

pub struct CheckInstructionTask {
    dump_task: Task<()>,
    wait_task: Task<ResultIn<(), ProcessInstructions>>,
    check_task: Task<Vec<String>>,
}

impl CheckInstructionTask {
    pub fn wait(self) -> Result<Vec<String>, Error> {
        self.dump_task.wait();
        self.wait_task.wait()?;
        let result = self.check_task.wait();
        Ok(result)
    }
}

/// Parse the output of objdump -T
fn process_objdump_syms(
    id: &str,
    raw_symbols: impl IntoIterator<Item = impl AsRef<str>>,
    send: mpsc::Sender<String>,
) -> Result<(), Error> {
    verboseln!("loading {}", id);
    let mut iter = raw_symbols.into_iter();
    for line in iter.by_ref() {
        if line.as_ref() == "DYNAMIC SYMBOL TABLE:" {
            break;
        }
    }

    // Example
    // # 0000000000000000      DF *UND*	0000000000000000 nnsocketGetPeerName
    //                   ^ spaces      ^ this is a tag

    for line in iter {
        let line = line.as_ref();
        if line.len() <= 25 {
            continue;
        }
        let symbol = match line[25..].split_once(' ').map(|x| x.1) {
            Some(symbol) => symbol,
            None => {
                let err = report!(Error::ParseSymbols(id.to_string(),))
                    .attach_printable(format!("invalid line: {}", line));
                return Err(err);
            }
        };
        send.send(symbol.to_string()).unwrap();
    }

    verboseln!("loaded '{}'", id);
    Ok(())
}

/// Parse the output of objdump --disassemble
///
/// Returns a list of (address, instructions)
fn process_objdump_insts(
    raw_instructions: impl IntoIterator<Item = impl AsRef<str>>,
    send: mpsc::Sender<(String, String)>,
) {
    raw_instructions
        .into_iter()
        .flat_map(|line| {
            let line = line.as_ref();
            // Example
            // 0000000000000000 <__code_start__>:
            //        0:	14000008 	b	20 <entrypoint>
            //        4:	0001a6e0 	.word	0x0001a6e0
            //        8:	d503201f 	nop
            //          ^ tab       _^ tab
            let mut parts = line.splitn(2, ":\t");
            let addr = parts.next()?.to_string();
            let bytes_and_asm = parts.next()?;
            //14000008 	b	20 <entrypoint>
            let (_bytes, inst) = bytes_and_asm.split_once(" \t")?;
            //b	20 <entrypoint>
            Some((addr, inst.to_string()))
        })
        .for_each(|inst| {
            send.send(inst).unwrap();
        });
}
