// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use cu::pre::*;
use regex::Regex;

use crate::env::environment;

type Symbols = HashSet<String>;

pub async fn check_all(
    elf: &Path,
    ignored_symbols: &[String],
    disallowed_instructions: &[String],
    symbol_files: &[PathBuf],
) -> cu::Result<()> {
    let progress = cu::progress("Checking ELF").spawn();
    let expected_symbols = load_known_symbols(symbol_files)?;

    let (missing_symbols, disallowed_instructions) = cu::co::try_join!(
        check_symbols(elf, expected_symbols, ignored_symbols),
        check_instructions(elf, disallowed_instructions)
    )?;

    if !missing_symbols.is_empty() {
        return Err(cu::fmterr!(
            "Missing symbols in {}:\n{:#?}",
            elf.display(),
            missing_symbols
        ));
    } else {
        cu::debug!("Check: no missing symbols")
    }
    if !disallowed_instructions.is_empty() {
        return Err(cu::fmterr!(
            "Found disallowed instructions in {}:\n{:#?}",
            elf.display(),
            disallowed_instructions
        ));
    } else {
        cu::debug!("Check: no disallowed instructions")
    }
    progress.done();

    Ok(())
}

/// Expects canonical paths
fn load_known_symbols(symbol_files: &[PathBuf]) -> cu::Result<Symbols> {
    let mut symbols = HashSet::new();

    for symbol_file in symbol_files {
        let content = cu::fs::read_string(symbol_file).context(format!(
            "failed to read symbol file {}",
            &symbol_file.display()
        ))?;
        let syms = parse_objdump_syms(content);
        symbols.extend(syms);
    }

    Ok(symbols)
}

async fn check_symbols(
    elf: &Path,
    expected_symbols: Symbols,
    ignored_symbols: &[String],
) -> cu::Result<Vec<String>> {
    let (child, stdout_handle) = environment()
        .objdump()
        .command()
        .arg("-T")
        .arg(elf)
        .stdout(cu::pio::string())
        .stdie_null()
        .co_spawn()
        .await?;

    child.co_wait_nz().await?;
    let stdout = stdout_handle.co_join().await??;
    let mut symbols = parse_objdump_syms(stdout);

    cu::debug!("symbols: {:#?}", symbols);

    for ignored_symbol in ignored_symbols {
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

fn parse_objdump_syms(content: String) -> Symbols {
    let mut lines = content.lines();

    for line in lines.by_ref() {
        if line == "DYNAMIC SYMBOL TABLE:" {
            break;
        }
    }

    lines
        .filter_map(|line| {
            if line.len() <= 25 {
                None
            } else {
                line[25..]
                    .split_once(' ')
                    .map(|x| x.1)
                    .map(|sym| sym.to_owned())
            }
        })
        .collect()
}

async fn check_instructions(
    elf: &Path,
    disallowed_instructions: &[String],
) -> cu::Result<Vec<String>> {
    let (child, stdout_handle) = environment()
        .objdump()
        .command()
        .arg("-d")
        .arg(elf)
        .stdout(cu::pio::string())
        .stdie_null()
        .co_spawn()
        .await?;

    child.co_wait_nz().await?;
    let stdout = stdout_handle.co_join().await??;
    let instructions = parse_objdump_insts(stdout);

    let mut disallowed_regexes = vec![
        Regex::new(r"^msr\s*spsel").unwrap(),
        Regex::new(r"^msr\s*daifset").unwrap(),
        Regex::new(r"^mrs\.*daif").unwrap(),
        Regex::new(r"^mrs\.*tpidr_el1").unwrap(),
        Regex::new(r"^msr\s*tpidr_el1").unwrap(),
        Regex::new(r"^hlt").unwrap(),
    ];
    disallowed_regexes.extend(disallowed_instructions.iter().filter_map(|inst| {
        Regex::new(inst)
            .inspect_err(|e| {
                cu::error!(
                    "Failed to parse disallowed instruction {}. Error: {}",
                    inst,
                    e
                )
            })
            .ok()
    }));

    let bad_instructions: Vec<String> = instructions
        .iter()
        .filter(|inst| disallowed_regexes.iter().any(|r| r.is_match(&inst.1)))
        .map(|bad_inst| format!("{}: {}", bad_inst.0, bad_inst.1))
        .collect();

    Ok(bad_instructions)
}

fn parse_objdump_insts(content: String) -> Vec<(String, String)> {
    // Example
    // 0000000000000000 <__code_start__>:
    //        0:	14000008 	b	20 <entrypoint>
    //        4:	0001a6e0 	.word	0x0001a6e0
    //        8:	d503201f 	nop
    //          ^ tab       _^ tab
    content
        .lines()
        .flat_map(|line| {
            let (addr, bytes_and_asm) = line.split_once(":\t")?;
            let (_bytes, inst) = bytes_and_asm.split_once(" \t")?;
            Some((addr.to_string(), inst.to_string()))
        })
        .collect()
}
