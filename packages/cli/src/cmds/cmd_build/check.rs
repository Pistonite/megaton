// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{collections::BTreeSet, path::PathBuf};

use cu::{PathExtension, Spawn};

use crate::{cmds::cmd_build::{BTArtifacts, config::Check}};


type Symbols = BTreeSet<String>;

pub fn check_symbols(elf_path: &PathBuf, expected_symbols: Symbols, check_config: &Check) -> cu::Result<Vec<String>>{
    let (child, stdout_handle) = cu::which("objdump")?.command()
        .arg("-T").arg(elf_path)
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;

    child.wait()?;

    let stdout = stdout_handle.join()??;
    
    let mut symbols = parse_objdump_syms(stdout);
    for ignored_symbol in &check_config.ignore {
        symbols.remove(ignored_symbol);
    }

    let missing_symbols = symbols
            .into_iter()
            .filter(|symbol| {
                // dot is not a valid character in a C identifier, most likely a false positive (.data, .text)
                !symbol.starts_with(".") && !expected_symbols.contains(symbol)
            }) 
            .collect::<Vec<_>>();

    Ok(missing_symbols)
}

pub fn load_known_symbols(btart: &BTArtifacts, config: &Check) -> cu::Result<Symbols> {
    let mut symbols = BTreeSet::new();
    let symbol_files = config.symbols.iter().filter_map(|p| {
        let symbol_file_path = btart.project_root.join(p);
        symbol_file_path.canonicalize()
            .inspect_err(|e| cu::error!("failed to read symbol file {}. error: {}", symbol_file_path.display(), e))
            .ok()
    });
    
    for symbol_file in symbol_files {
        if let Ok(content) = cu::fs::read_string(&symbol_file)
            .inspect_err(|e| cu::error!("failed to read symbol file {}: error: {}", &symbol_file.display(), e)) {
                let syms = parse_objdump_syms(content);
                symbols.extend(syms);
        }
    }

    Ok(symbols)
}

fn parse_objdump_syms(content: String) -> BTreeSet<String> {
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        if line == "DYNAMIC SYMBOL TABLE:" {
            break;
        }
    }

    lines.filter_map(|line| {
        if line.len() <= 25 {
            None
        } else  {
            line[25..].splitn(2, ' ').skip(1).next().map(|sym| sym.to_owned())
        }
    }).collect()

}

fn parse_objdump_insts(output: String) -> Vec<(String,String)> {
    // Example
    // 0000000000000000 <__code_start__>:
    //        0:	14000008 	b	20 <entrypoint>
    //        4:	0001a6e0 	.word	0x0001a6e0
    //        8:	d503201f 	nop
    //          ^ tab       _^ tab
    output.lines().flat_map(|line| {
        let mut parts = line.splitn(2, ":\t");
        let addr = parts.next()?.to_string();
        let bytes_and_asm = parts.next()?;
        let mut parts = bytes_and_asm.splitn(2, " \t");
        let _bytes = parts.next()?;
        //14000008 	b	20 <entrypoint>
        let inst = parts.next()?;
        //b	20 <entrypoint>
        Some((addr, inst.to_string()))
    }).collect()
}
