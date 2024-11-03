//! The megaton build command

use std::io::BufRead;
use std::path::PathBuf;
use std::time::Instant;

use buildcommon::system::PathExt;
use buildcommon::{system, errorln, hintln, infoln, verboseln};
use error_stack::ResultExt;
use filetime::FileTime;
use rustc_hash::FxHashMap;
use serde_json::{json, Value};
use walkdir::WalkDir;

use crate::build::{
    load_checker, load_compile_commands, BuildResult, BuildTask, Builder, Config, Paths,
    SourceResult,
};
use crate::system::{ChildBuilder, Error, Executer};
use crate::Options;

use super::config::Check;

/// Run megaton build
pub fn run(dir: &str, options: &Options) -> Result<(), Error> {
    let start_time = Instant::now();

    let root = crate::system::find_root(dir).map_err(Error::InteropSelf)?;
    let megaton_toml = root.join("Megaton.toml");
    let config = Config::from_path(&megaton_toml)?;
    let profile = match (options.profile.as_str(), &config.module.default_profile) {
        ("none", Some(p)) if p.is_empty() => {
            // default-profile = "" means to disallow no profile
            return Err(Error::NoProfile);
        }
        ("none", Some(p)) => p,
        ("none", None) => "none",
        (profile, _) => profile,
    };

    let paths = Paths::new(root, profile, &config.module.name)?;

    let executer = Executer::new();

    let mut main_npdm_task = None;

    infoln!("Building", "{} (profile `{profile}`)", config.module.name);
    system::ensure_directory(&paths.target_o).map_err(Error::Interop)?;
    let megaton_toml_mtime = system::get_mtime(&megaton_toml).map_err(Error::Interop)?;
    let npdm_json = paths.target.join("main.npdm.json");
    let npdm_mtime = system::get_mtime(&npdm_json).map_err(Error::Interop)?;
    let megaton_toml_changed = !system::up_to_date(megaton_toml_mtime, npdm_mtime);

    if megaton_toml_changed {
        let target = paths.target.clone();
        let npdmtool = paths.npdmtool.clone();
        let title_id = config.module.title_id_hex();
        let task = executer.execute(move || {
            infoln!("Creating", "main.npdm");
            // unwrap: megaton.toml must exist
            create_npdm(target, npdmtool, title_id, megaton_toml_mtime.unwrap())?;
            verboseln!("created main.npdm");
            Ok::<(), Error>(())
        });

        main_npdm_task = Some(task);
    }

    let build = config.build.get_profile(profile);
    let entry = build.entry.as_ref().ok_or(Error::NoEntryPoint)?;

    let cc_possibly_changed = megaton_toml_changed;
    let mut compile_commands = FxHashMap::default();
    let mut new_compile_commands = Vec::new();
    if cc_possibly_changed {
        // even though this is blocking
        // this will only load when Megaton.toml changes
        load_compile_commands(&paths.cc_json, &mut compile_commands);
    }
    let builder = Builder::new(&paths, entry, &build)
        .change_context(Error::CreateBuilder)
        .map_err(Error::InteropSelf)?;
    // if any .o files were rebuilt
    let mut objects_changed = false;
    // all .o files
    let mut objects = Vec::new();
    let mut cc_tasks = Vec::new();

    // fire off all cc tasks
    for source_dir in &build.sources {
        let source_dir = paths.root.join(source_dir);
        for entry in WalkDir::new(source_dir).into_iter().flatten() {
            let source_path = entry.path();
            let cc =
                builder.process_source(source_path, cc_possibly_changed, &mut compile_commands)
                .map_err(Error::Interop)?;
            let cc = match cc {
                SourceResult::NotSource => {
                    // file type not recognized, skip
                    continue;
                }
                SourceResult::UpToDate(o_file) => {
                    verboseln!(
                        "skipped '{}'",
                        source_path.rebase(&paths.root).display()
                    );
                    objects.push(o_file);
                    continue;
                }
                SourceResult::NeedCompile(cc) => cc,
            };
            objects_changed = true;
            objects.push(cc.output.clone());
            let source_display = source_path.rebase(&paths.root).display().to_string();
            let child = cc.create_child();
            let task = executer.execute(move || {
                infoln!("Compiling", "{}", source_display);
                let child = BuildTask::new(child.spawn()?);
                let result = child.wait()?;
                if !result.success {
                    verboseln!("failed to build '{}'", source_display);
                } else {
                    verboseln!("built '{}'", source_display);
                }
                Ok::<BuildResult, Error>(result)
            });
            new_compile_commands.push(cc);
            cc_tasks.push(task);
        }
    }

    let verfile_task = if megaton_toml_changed {
        let verfile = paths.verfile.clone();
        let entry = entry.clone();
        Some(executer.execute(move || {
            verboseln!("creating verfile");
            create_verfile(verfile, entry)?;
            infoln!("Created", "verfile");
            Ok::<(), Error>(())
        }))
    } else {
        None
    };

    // if compiled, save cc_json
    let save_cc_json_task = if objects_changed || !compile_commands.is_empty() {
        let file = system::buf_writer(&paths.cc_json).map_err(Error::Interop)?;
        let path_display = paths.cc_json.display().to_string();
        Some(executer.execute(move || {
            verboseln!("saving compile_commands.json");
            serde_json::to_writer_pretty(file, &new_compile_commands)
                .map_err(|e| Error::ParseJson(path_display, e))?;
            verboseln!("saved compile_commands.json");
            Ok::<(), Error>(())
        }))
    } else {
        None
    };

    // compute if linking is needed

    // compile_commands not empty means sources were removed
    // link flags can change if megaton toml changed
    let mut needs_linking = objects_changed
        || !compile_commands.is_empty()
        || megaton_toml_changed
        || !paths.elf.exists();

    // LD scripts can change
    if !needs_linking {
        let elf_mtime = system::get_mtime(&paths.elf).map_err(Error::Interop)?;
        for ldscript in &build.ldscripts {
            let ldscript = paths.root.join(ldscript);
            let mtime = system::get_mtime(&ldscript).map_err(Error::Interop)?;
            if !system::up_to_date(mtime, elf_mtime) {
                needs_linking = true;
                break;
            }
        }
    }
    // objects can be newer than elf even if not changed
    // note that even if compile is in progress, this works
    if !needs_linking {
        let elf_mtime = system::get_mtime(&paths.elf).map_err(Error::Interop)?;
        for object in &objects {
            // get_mtime can error if obj is being compiled
            let mtime = match system::get_mtime(object) {
                Ok(mtime) => mtime,
                Err(_) => {
                    needs_linking = true;
                    break;
                }
            };
            if !system::up_to_date(mtime, elf_mtime) {
                needs_linking = true;
                break;
            }
        }
    }
    // TODO: libs can change

    let check_config = config.check.as_ref().map(|c| c.get_profile(profile));

    let mut needs_nso = needs_linking || !paths.nso.exists();
    // symbol files can change
    if !needs_nso {
        if let Some(config) = check_config.as_ref() {
            let nso_mtime = system::get_mtime(&paths.nso).map_err(Error::Interop)?;
            needs_nso = symbol_listing_changed(config, &paths, nso_mtime)?;
        }
    }
    // elf can be newer if check failed
    if !needs_nso {
        // note we don't need to wait for linker here
        // because if is linking -> needs_linking must be true
        let elf_mtime = system::get_mtime(&paths.elf).map_err(Error::Interop)?;
        let nso_mtime = system::get_mtime(&paths.nso).map_err(Error::Interop)?;
        if !system::up_to_date(elf_mtime, nso_mtime) {
            needs_nso = true;
        }
    }

    // eagerly load checker if checking is needed
    let checker = match (needs_nso || needs_linking, check_config) {
        (true, Some(check)) => Some(load_checker(&paths, check, &executer)?),
        _ => None,
    };

    // start joining the cc tasks
    let mut compile_failed = false;
    for t in cc_tasks {
        match t.wait() {
            Err(e) => {
                errorln!("Error", "{}", e);
                compile_failed = true;
            }
            Ok(result) => {
                if !result.success {
                    compile_failed = true;
                }
                if let Some(error) = result.error {
                    for line in error.lines().map_while(Result::ok) {
                        errorln!("Error", "{}", line);
                    }
                }
            }
        }
    }
    if compile_failed {
        return Err(Error::CompileError);
    }

    // linker dependencies
    if needs_linking {
        if let Some(verfile_task) = verfile_task {
            verfile_task.wait()?;
        }
    }

    let elf_name = format!("{}.elf", config.module.name);

    let link_task = if needs_linking {
        let task = builder.link_start(&objects, &paths.elf)?;
        let elf_name = elf_name.clone();
        let task = executer.execute(move || {
            infoln!("Linking", "{}", elf_name);
            let result = task.wait()?;
            verboseln!("linked '{}'", elf_name);
            Ok::<BuildResult, Error>(result)
        });
        Some(task)
    } else {
        None
    };

    // nso dependency
    if let Some(task) = link_task {
        let result = task.wait()?;
        if !result.success {
            if let Some(error) = result.error {
                for line in error.lines().map_while(Result::ok) {
                    errorln!("Error", "{}", line);
                }
            }
            return Err(Error::LinkError);
        }
    }

    if needs_nso {
        let nso_name = format!("{}.nso", config.module.name);
        let mut checker = checker.expect("checker not loaded, dependency bug");
        infoln!("Checking", "{}", elf_name);
        let missing_symbols = checker.check_symbols(&executer)?;
        let bad_instructions = checker.check_instructions(&executer)?;
        let missing_symbols = missing_symbols.wait()?;
        let bad_instructions = bad_instructions.wait()?;
        let mut check_ok = true;
        if !missing_symbols.is_empty() {
            errorln!("Error", "There are unresolved symbols:");
            errorln!("Error", "");
            for symbol in missing_symbols.iter().take(10) {
                errorln!("Error", "  {}", symbol);
            }
            if missing_symbols.len() > 10 {
                errorln!("Error", "  ... ({} more)", missing_symbols.len() - 10);
            }
            errorln!("Error", "");
            errorln!(
                "Error",
                "Found {} unresolved symbols!",
                missing_symbols.len()
            );
            let missing_symbols = missing_symbols.join("\n");
            let missing_symbols_path = paths.target.join("missing_symbols.txt");
            system::write_file(&missing_symbols_path, &missing_symbols).map_err(Error::Interop)?;
            hintln!(
                "Hint",
                "Include the symbols in the linker scripts, or add them to the `ignore` section."
            );
            hintln!(
                "Saved",
                "All missing symbols to `{}`",
                paths.from_root(missing_symbols_path).display()
            );
            check_ok = false;
        }
        if !bad_instructions.is_empty() {
            errorln!("Error", "There are unsupported/disallowed instructions:");
            errorln!("Error", "");
            for inst in bad_instructions.iter().take(10) {
                errorln!("Error", "  {}", inst);
            }
            if bad_instructions.len() > 10 {
                errorln!("Error", "  ... ({} more)", bad_instructions.len() - 10);
            }
            errorln!("Error", "");
            errorln!(
                "Error",
                "Found {} disallowed instructions!",
                bad_instructions.len()
            );

            let output = bad_instructions.join("\n");
            let output_path = paths.target.join("disallowed_instructions.txt");
            // TODO: error
            system::write_file(&output_path, &output).map_err(Error::Interop)?;
            hintln!(
                "Saved",
                "All disallowed instructions to {}",
                paths.from_root(output_path).display()
            );
            check_ok = false;
        }
        if !check_ok {
            return Err(Error::CheckError);
        }
        hintln!("Checked", "Looks good to me");

        // create the nso after checking
        // so if check failed, nso is not created,
        // and an immediately build with no change won't succeed
        infoln!("Creating", "{}", nso_name);

        let status = ChildBuilder::new(&paths.elf2nso)
            .args([&paths.elf, &paths.nso])
            .silent()
            .spawn()?
            .wait()?;
        if !status.success() {
            return Err(Error::Elf2NsoError);
        }
    }

    if let Some(task) = save_cc_json_task {
        task.wait()?;
    }

    if let Some(task) = main_npdm_task {
        task.wait()?;
    }

    let elapsed = start_time.elapsed();
    infoln!(
        "Finished",
        "{} (profile `{profile}`) in {:.2}s",
        config.module.name,
        elapsed.as_secs_f32()
    );

    Ok(())
}

fn create_npdm(
    target: PathBuf,
    npdmtool: PathBuf,
    title_id: String,
    m_time: FileTime,
) -> Result<(), Error> {
    let mut npdm_data: Value =
        serde_json::from_str(include_str!("../../template/main.npdm.json")).unwrap();
    npdm_data["title_id"] = json!(format!("0x{}", title_id));
    let npdm_data = serde_json::to_string_pretty(&npdm_data).expect("fail to serialize npdm data");
    let npdm_json = target.join("main.npdm.json");
    system::write_file(&npdm_json, &npdm_data).map_err(Error::Interop)?;
    system::set_mtime(&npdm_json, m_time).map_err(Error::Interop)?;
    let main_npdm = target.join("main.npdm");
    let npdm_status = ChildBuilder::new(npdmtool)
        .args(crate::system::args![&npdm_json, &main_npdm])
        .silent()
        .spawn()?
        .wait()?;
    if !npdm_status.success() {
        return Err(Error::NpdmError(npdm_status));
    }
    Ok(())
}

fn create_verfile(verfile: PathBuf, entry: String) -> Result<(), Error> {
    let verfile_data = format!(
        "{}{}{}",
        include_str!("../../template/verfile.before"),
        entry,
        include_str!("../../template/verfile.after")
    );
    system::write_file(verfile, &verfile_data).map_err(Error::Interop)?;
    Ok(())
}

pub fn clean(dir: &str, options: &Options) -> Result<(), Error> {
    let root = crate::system::find_root(dir).map_err(Error::InteropSelf)?;
    let mut target = root.clone();
    target.push("target");
    target.push("megaton");
    if "none" != &options.profile {
        target.push(&options.profile);
    }
    if !root.exists() {
        hintln!("Skipped", "{}", target.rebase(&root).display());
        return Ok(());
    }

    system::remove_directory(&target).map_err(Error::Interop)?;
    infoln!("Cleaned", "{}", target.rebase(&root).display());
    Ok(())
}

fn symbol_listing_changed(config: &Check, paths: &Paths, m_time: Option<FileTime>) -> Result<bool, Error> {
    for symbol in &config.symbols {
        let symbol = paths.root.join(symbol);
        let symbol_mtime = system::get_mtime(&symbol).map_err(Error::Interop)?;
        if !system::up_to_date(symbol_mtime, m_time) {
            return Ok(true);
        }
    }
    Ok(false)
}
