use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cu::pre::*;
use tokio::sync::mpsc;

use megaton_config::toolchain::ToolchainEnv;
use megaton_config::config_file::Config;

pub struct Checker {
    defined_dynamic_symbols: BTreeSet<String>
}

impl Checker {
    #[cu::context("failed to load checker")]
    pub async fn load(config: &Config) -> cu::Result<Checker> {
        let (send, mut recv) = mpsc::unbounded_channel::<String>();
        let mut defined_dynamic_symbols = BTreeSet::new();
        let check_config = config.check_config()?;
        defined_dynamic_symbols.extend(check_config.symbols.iter().cloned());

        if !check_config.symbol_files.is_empty() {
            let pool = cu::co::pool(-2);
            let mut handles = Vec::with_capacity(check_config.symbol_files.len());

            for file in &check_config.symbol_files {
                let send = send.clone();
                let file = file.clone();
                let handle = pool.spawn(async move {
                    load_symbol_list(&file, send).await
                });
                handles.push(handle);
            }
            drop(send);

            while let Some(x) = recv.recv().await {
                defined_dynamic_symbols.insert(x);
            }

            for handle in handles {
                handle.co_join().await??;
            }
        }

        Ok(Checker { defined_dynamic_symbols })
    }

    /// Return if the ELF is fresh, meaning the NSO is not up-to-date
    pub async fn check(self, 
        config: &Config,
        toolchain: &ToolchainEnv,
        mut should_check: bool,
    ) -> cu::Result<bool> {
        let elf_path = &config.project.out_elf;
        if !should_check {
            let nso_path = &config.project.out_nso;
            // only check if up-to-date if we are not already going to run check anyway
            let up_to_date = 
            is_elf_check_up_to_date(elf_path, nso_path, &config.project.config_path)
        ?;
            if !up_to_date {
                should_check = true;
            }
        }

        if !should_check {
            return Ok(false);
        }

        cu::debug!("running elf checks...");
        let objdump = &toolchain.devkita64.objdump;

        let undefined_symbols = {
            let objdump = PathBuf::from(objdump);
            let elf_path = elf_path.clone();
            let defined_dynamic_symbols = self.defined_dynamic_symbols;
            cu::co::spawn(async move {
                get_undefined_symbols(
                    &objdump,
                    &defined_dynamic_symbols,
                    &elf_path,
                ).await
            })
        };

        let disallowed_instructions_errors = {
            let objdump = PathBuf::from(objdump);
            let elf_path = elf_path.clone();
            let disallowed_instructions = config.check_config()?.disallowed_instructions.clone();
            cu::co::spawn(async move {
                check_disallowed_instructions(
                    &objdump,
                    &elf_path,
                    &disallowed_instructions
                ).await
            })
        };

        let mut has_error = false;

        match undefined_symbols.co_join().await.flatten() {
            Err(e) => {
                cu::error!("undefined symbols check failed: {e:?}");
                has_error =true;
            }
            Ok(undefined_symbols) => {
                if !undefined_symbols.is_empty() {
                    report_check_errors(
                        config,
                        &undefined_symbols,
                        "undefined_symbols.txt",
                        "undefined dynamic symbols",
                        10,
                    );
                    has_error = true;
                } else {
                    cu::debug!("undefined dynamic symbols check passed");
                }
            }
        }

        match disallowed_instructions_errors.co_join().await.flatten() {
            Err(e) => {
                cu::error!("disallowed instructions check failed: {e:?}");
                has_error =true;
            }
            Ok(errors) => {
                if !errors.is_empty() {
                    report_check_errors(
                        config,
                        &errors,
                        "disallowed_instructions.txt",
                        "disallowed instructions",
                        10,
                    );
                    has_error = true;
                } else {
                    cu::debug!("disallowed instructions check passed");
                }
            }
        }

        if has_error {
            cu::bail!("elf checks failed; please see errors above");
        }

        cu::hint!("{}.elf looks good to me", config.project.module_name);
        Ok(true)
    }
}


#[cu::context("failed to check if elf check is up-to-date")]
fn is_elf_check_up_to_date(
    elf_path: &Path,
    nso_path: &Path,
    config_path: &Path,
) -> cu::Result<bool> {
    let nso_mtime = match cu::fs::get_mtime(nso_path) {
        Err(e) => {
            cu::debug!("need to run check: nso is not accessible: {e:?}");
            return Ok(false);
        }
        Ok(None) => {
            cu::debug!("need to run check: nso not found");
            return Ok(false);
        }
        Ok(Some(x)) => x
    };

    let Some(elf_mtime) = cu::fs::get_mtime(elf_path)? else {
        cu::bail!("elf not found: {}", elf_path.display());
    };

    if elf_mtime > nso_mtime {
        cu::debug!("need to run check: elf is fresh");
        return Ok(false);
    }

    if let Ok(Some(config_mtime)) = cu::fs::get_mtime(config_path) {
        // always run check if config file changed, since that's where the checks
        // are defined
        if config_mtime > nso_mtime {
            cu::debug!("need to run check: config file changed");
            return Ok(false);
        }
    }

    cu::debug!("elf check is up-to-date");
    return Ok(true)
}

#[cu::context("failed to load symbol file '{}'", path.display())]
async fn load_symbol_list(path: &Path, sender: mpsc::UnboundedSender<String>) -> cu::Result<()> {
    let reader = cu::fs::co_reader(path).await?;
    let mut lines = reader.lines();

    let mut is_objdump_format = false;

    cu::debug!("parsing symbol file {}", path.display());
    let mut count = 0;

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("DYNAMIC SYMBOL TABLE") || line.contains("file format ") || line.contains("no symbols"){
            is_objdump_format = true;
            break;
        }
        if line.starts_with('0') {
            if let Some(symbol) = parse_line(line, is_objdump_format) {
                count += 1;
                sender.send(symbol.to_string())?;
            }
            break;
        }
    }

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(symbol) = parse_line(line, is_objdump_format) {
            count += 1;
            sender.send(symbol.to_string())?;
        }
    }

    cu::debug!("parsed {count} symbols from {}", path.display());

    Ok(())
}

/// Check each undefined symbol in the ELF is a defined dynamic symbol
#[cu::context("failed to run undefined dynamic symbols check")]
async fn get_undefined_symbols(
    objdump: &Path,
    defined_dynamic_symbols: &BTreeSet<String>,
    elf_path: &Path
) -> cu::Result<Vec<String>> {
    let (child, mut stdout) = 
    objdump.command()
    .add(cu::args!["-T", elf_path])
        .stdout(cu::pio::co_lines())
        .stderr(cu::lv::E)
        .stdin_null()
        .co_spawn().await?;

    let mut undefined_dynamic = vec![];
    let mut has_error = false;
    while let Some(line) = stdout.next().await {
        let line = match line {
            Ok(x) => x,
            Err(e) => {
                cu::error!("error reading objdump -T output: {e:?}");
                has_error = true;
                break;
            }
        };
        let line = line.trim();
        if line.starts_with("DYNAMIC SYMBOL TABLE") || line.contains("file format ")|| line.contains("no symbols"){
            continue;
        }
        let Some(symbol) = parse_line(line, true) else {
            continue;
        };
        if symbol.starts_with('.') {
            // ignore section labels
            continue;
        }

        if !defined_dynamic_symbols.contains(symbol) {
            undefined_dynamic.push(symbol.to_string());
        }
    }

    if has_error {
        cu::warn!("io error occured; check result might be false");
    }

    child.co_wait_nz().await?;
    Ok(undefined_dynamic)
}

fn parse_line(line: &str, is_objdump_format: bool) -> Option<&str> {
    if is_objdump_format {
        if let Some(symbol) = parse_objdump_format_line(line) {
            return Some(symbol)
        }parse_llvm_nm_format_line(line)
    } else {
        if let Some(symbol) = parse_llvm_nm_format_line(line) {
            return Some(symbol)
        }
        parse_objdump_format_line(line)
    }
}

fn parse_llvm_nm_format_line(line: &str) -> Option<&str> {
    if line.trim().starts_with('U') {
        // undefined symbol
        return None;
    }
    let i = line.rfind(' ')?;
    let sym = line[i+1..].trim();
    if sym.is_empty() {
        return None;
    }
    Some(sym)
}
fn parse_objdump_format_line(line: &str) -> Option<&str> {
    if line.contains("*UND*") {
        // undefined symbol
        return None;
    }
    let i = line.rfind(' ')?;
    let sym = line[i+1..].trim();
    if sym.is_empty() {
        return None;
    }
    Some(sym)
}

#[cu::context("failed to run disallowed instructions check")]
async fn check_disallowed_instructions(
    objdump: &Path,
    elf_path: &Path,
    disallowed_instructions: &[cu::Regex]
) -> cu::Result<Vec<String>> {
    let (child, mut stdout) = 
    objdump.command()
    .add(cu::args!["-d", elf_path])
        .stdout(cu::pio::co_lines())
        .stderr(cu::lv::E)
        .stdin_null()
        .co_spawn().await?;

    let mut disallowed_instructions_errors = vec![];
    let mut has_error = false;
        while let Some(line) = stdout.next().await {
            let line = match line {
                Ok(x) => x,
                Err(e) => {
                    cu::error!("error reading objdump -d output: {e:?}");
                    has_error = true;
                    break;
                }
            };
            let Some((addr, insn)) = parse_objdump_assembly(&line) else {
                continue;
            };

        for pattern in disallowed_instructions {
            if pattern.is_match(insn) {
                disallowed_instructions_errors.push(format!("{addr}: {insn} // pattern: {pattern}"));
            }
        }
        }
    if has_error {
        cu::warn!("io error occured; check result might be false");
    }

    child.co_wait_nz().await?;
    Ok(disallowed_instructions_errors)
}

fn parse_objdump_assembly(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let (addr, bytes_and_asm) = line.split_once(":\t")?;
    let (_bytes, inst) = bytes_and_asm.split_once(" \t")?;
    Some((addr, inst))
}

/// Print the errors of a failed check and save the full list to `report_file`
/// in the output directory.
///
/// If there are more than `truncate_threshold` errors, the printed list is
/// truncated to fit in that many lines, including the "... and N more" line.
fn report_check_errors(
    config: &Config,
    errors: &[String],
    report_file: &str,
    description: &str,
    truncate_threshold: usize,
) {
    fn push_errors(report: &mut String, errors: &[String]) {
        for s in errors {
            report.push_str("    ");
            report.push_str(s);
            report.push('\n');
        }
    }

    let header = format!("\n  CHECK FAILED: there are {description}!\n");

    let mut full_report = header.clone();
    push_errors(&mut full_report, errors);
    full_report.push('\n');

    let report_path = config.project.output_root.join(report_file);
    if let Err(e) = cu::fs::write(&report_path, &full_report) {
        cu::warn!("failed to save {report_file}: {e:?}");
    }

    if errors.len() > truncate_threshold {
        let shown = truncate_threshold.saturating_sub(1);
        let mut short_report = header;
        push_errors(&mut short_report, &errors[..shown]);
        short_report.push_str(&format!("    ... and {} more\n\n", errors.len() - shown));
        cu::error!("{short_report}");
    } else {
        cu::error!("{full_report}");
    }

    cu::hint!(
        "the full list of {description} is saved to {}",
        report_path.try_to_rel_from(&config.project.root).display()
    );
}
