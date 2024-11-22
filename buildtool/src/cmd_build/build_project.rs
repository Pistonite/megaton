//! The megaton build command
use buildcommon::compdb::CompileDB;
use buildcommon::prelude::*;

use std::path::{Path, PathBuf};

use buildcommon::env::ProjectEnv;
use buildcommon::system::{Command, Executor};
use filetime::FileTime;
use serde_json::{json, Value};
use walkdir::WalkDir;

use super::builder::{Builder, SourceResult};
use super::checker;
use super::config::{Check, Config};
use super::Options;

use crate::error::Error;

/// Run megaton build
///
/// Return the name of the built artifact
pub fn run(
    env: &ProjectEnv,
    config: &Config,
    profile: &str,
    megaton_toml: &Path,
    options: &Options,
) -> Result<String, Error> {
    let executor = Executor::new();

    let mut main_npdm_task = None;

    infoln!("Building", "{} (profile `{}`)", config.module.name, profile);
    system::ensure_directory(&env.target_o).change_context(Error::BuildPrep)?;
    let megaton_toml_mtime = system::get_mtime(megaton_toml).change_context(Error::BuildPrep)?;
    let npdm_json = env.target.join("main.npdm.json");
    let npdm_mtime = system::get_mtime(&npdm_json).change_context(Error::BuildPrep)?;
    let megaton_toml_changed = !system::up_to_date(megaton_toml_mtime, npdm_mtime);

    verboseln!("megaton.toml changed: {}", megaton_toml_changed);

    if megaton_toml_changed && !options.compdb {
        let target = env.target.clone();
        let npdmtool = env.npdmtool.clone();
        let title_id = config.module.title_id_hex();
        let task = executor.execute(move || {
            // unwrap: megaton.toml must exist
            create_npdm(target, npdmtool, title_id, megaton_toml_mtime.unwrap())
        });

        main_npdm_task = Some(task);
    }

    let build = config.build.get_profile(profile);

    // if compdb should be checked for if an edge needs rebuilding
    // when --compdb, we force regenerate all compile commands
    let check_compdb = megaton_toml_changed || options.compdb;
    let mut compdb = if check_compdb {
        CompileDB::load(&env.compdb_cache)
    } else {
        Default::default()
    };
    let mut builder = Builder::new(env, &config.module, &build)?;
    // if any .o files were rebuilt
    let mut objects_changed = false;
    // all .o files
    let mut objects = Vec::new();
    let mut cc_tasks = Vec::new();

    // fire off all cc tasks
    for source_dir in &builder.source_dirs {
        for entry in WalkDir::new(source_dir).into_iter().flatten() {
            let source_path = entry.path();
            let cc = builder
                .process_source(source_path, check_compdb, &compdb)
                .change_context(Error::SourcePrep)?;
            let child = match cc {
                SourceResult::NotSource => {
                    // file type not recognized, skip
                    continue;
                }
                SourceResult::UpToDate(o_file) => {
                    verboseln!("skipped '{}'", env.from_root(source_path).display());
                    objects.push(o_file);
                    continue;
                }
                SourceResult::NeedCompile(cc, o_file) => {
                    let child = cc.create_command(&o_file);
                    compdb.insert(o_file.clone(), cc);
                    objects.push(o_file);
                    child
                }
            };
            objects_changed = true;
            let source_display = env.from_root(source_path).display().to_string();
            if !options.compdb {
                let task = executor.execute(move || {
                    infoln!("Compiling", "{}", source_display);
                    let child = child.spawn()?;
                    let result = child.wait()?;
                    if !result.is_success() {
                        verboseln!("failed to build '{}'", source_display);
                    } else {
                        verboseln!("built '{}'", source_display);
                    }
                    Ok::<_, Report<system::Error>>(result)
                });
                cc_tasks.push(task);
            }
        }
    }

    // keep order of objects.cache consistent
    // This allows O(n + n log n) instead of O(2n log n) when comparing
    objects.sort();

    let verfile_task = if megaton_toml_changed && !options.compdb {
        let verfile = env.verfile.clone();
        let entry = build.entry_point().to_string();
        Some(executor.execute(move || create_verfile(verfile, entry)))
    } else {
        None
    };

    if options.compdb {
        // since we didn't link, don't update compdb or objects cache
        // only update the compile_commands.json
        if !check_compdb {
            compdb.merge_old(&env.compdb_cache);
        }
        let mut file = system::buf_writer(&env.cc_json).change_context(Error::CompileDb)?;
        compdb
            .write_as_compile_commands(&env.system_headers, &mut file)
            .change_context(Error::CompileDb)?;
        return Ok(format!("compdb for profile `{}`", profile));
    }

    // if compiled, save cc_json
    let save_cc_json_task = if objects_changed || check_compdb {
        let path = env.compdb_cache.clone();
        let cc_json = env.cc_json.clone();
        let system_headers = env.system_headers.clone();
        let task = if check_compdb {
            executor.execute(move || compdb.save(&system_headers, &path, &cc_json))
        } else {
            executor.execute(move || {
                compdb.merge_old(&path);
                compdb.save(&system_headers, &path, &cc_json)
            })
        };
        Some(task)
    } else {
        None
    };

    // compute if linking is needed

    // link flags can change if megaton toml changed
    let mut needs_linking = objects_changed || megaton_toml_changed || !env.elf.exists();

    // The list of objects can change even if objects_changed is false,
    // due to sources being removed
    let new_obj_cache = format!("{}\n{}", objects.len(), objects.join("\n"));
    if !needs_linking {
        let old = system::read_file(&env.objects_cache).unwrap_or_default();
        if old != new_obj_cache {
            verboseln!("linking needed because list of objects changed");
            needs_linking = true;
        }
    }
    if let Err(e) = system::write_file(&env.objects_cache, &new_obj_cache) {
        errorln!("Error", "Failed to save objects cache: {}", e);
    }

    // LD scripts can change
    if !needs_linking {
        match system::get_mtime(&env.elf) {
            Ok(elf_mtime) => {
                for ldscript in &build.ldscripts {
                    let ldscript = env.root.join(ldscript);
                    let mtime = match system::get_mtime(&ldscript) {
                        Ok(mtime) => mtime,
                        Err(e) => {
                            errorln!(
                                "Failed",
                                "Cannot process linker script '{}'",
                                env.from_root(ldscript).display()
                            );
                            return Err(e).change_context(Error::Ldscript);
                        }
                    };
                    if !system::up_to_date(mtime, elf_mtime) {
                        needs_linking = true;
                        break;
                    }
                }
            }
            Err(e) => {
                needs_linking = true;
                verboseln!("failed to get mtime of elf: {}", e);
            }
        }
    }
    // objects can be newer than elf even if not recompiled
    // i.e. built in a previous run, but linking failed
    // note that even if compile is in progress, this works
    if !needs_linking {
        match system::get_mtime(&env.elf) {
            Ok(elf_mtime) => {
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
            Err(e) => {
                needs_linking = true;
                verboseln!("failed to get mtime of elf: {}", e);
            }
        }
    }
    // search for libs and see if they changed, etc..
    // this is somewhat expensive, but this only needs to be done
    // if linking is still not yet needed after all previous checks,
    if !needs_linking {
        match system::get_mtime(&env.elf) {
            Ok(elf_mtime) => {
                needs_linking = builder.check_lib_changed(&build, elf_mtime)?;
            }
            Err(e) => {
                needs_linking = true;
                verboseln!("failed to get mtime of elf: {}", e);
            }
        }
    }

    if needs_linking {
        builder.configure_linker(&build)?;
    }

    let check_config = config.check.as_ref().map(|c| c.get_profile(profile));

    let mut needs_nso = needs_linking || !env.nso.exists();
    // symbol files can change
    if !needs_nso {
        if let Some(config) = check_config.as_ref() {
            match system::get_mtime(&env.nso) {
                Err(e) => {
                    needs_nso = true;
                    verboseln!("failed to get mtime of nso: {}", e);
                }
                Ok(nso_mtime) => match symbol_listing_changed(config, env, nso_mtime) {
                    Err(e) => {
                        needs_nso = true;
                        verboseln!("failed to check if symbol listing changed: {}", e);
                    }
                    Ok(changed) => {
                        needs_nso = changed;
                    }
                },
            }
        }
    }
    // elf can be newer if check failed
    if !needs_nso {
        // note we don't need to wait for linker here
        // because if is linking -> needs_linking must be true
        let elf_mtime = system::get_mtime(&env.elf);
        let nso_mtime = system::get_mtime(&env.nso);
        match (elf_mtime, nso_mtime) {
            (Ok(elf_mtime), Ok(nso_mtime)) => {
                if !system::up_to_date(elf_mtime, nso_mtime) {
                    needs_nso = true;
                }
            }
            (elf_mtime, nso_mtime) => {
                if let Err(e) = elf_mtime {
                    verboseln!("failed to get mtime of elf: {}", e);
                }
                if let Err(e) = nso_mtime {
                    verboseln!("failed to get mtime of nso: {}", e);
                }
                needs_nso = true;
            }
        }
    }

    // eagerly load checker if checking is needed
    let checker = match (needs_nso || needs_linking, check_config) {
        (true, Some(check)) => {
            check.check();
            Some(checker::load(env, check, &executor)?)
        }
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
            Ok(mut result) => {
                if !result.is_success() {
                    result.dump_stderr("Error");
                    compile_failed = true;
                }
            }
        }
    }
    if compile_failed {
        errorln!("Error", "One or more object files failed to compile.");
        hintln!("Hint", "Please check the errors above.");
        let err = report!(Error::Compile).attach_printable("Please check the errors above.");
        return Err(err);
    }

    // linker dependencies
    if needs_linking {
        if let Some(verfile_task) = verfile_task {
            verfile_task.wait()?;
        }
    }

    let elf_name = format!("{}.elf", config.module.name);

    let link_task = if needs_linking {
        let task = builder.link_start(&objects).change_context(Error::Link)?;
        let elf_name = elf_name.clone();
        let task = executor.execute(move || {
            infoln!("Linking", "{}", elf_name);
            let result = task.wait().change_context(Error::Link)?;
            verboseln!("linked '{}'", elf_name);
            Ok::<_, Report<Error>>(result)
        });
        Some(task)
    } else {
        None
    };

    // nso dependency
    if let Some(task) = link_task {
        let mut child = task.wait()?;
        let result = child.check();
        if result.is_err() {
            child.dump_stderr("Error");
        }
        result.change_context(Error::Link)?;
    }

    if needs_nso {
        let nso_name = format!("{}.nso", config.module.name);
        if let Some(mut checker) = checker {
            infoln!("Checking", "{}", elf_name);
            let missing_symbols = checker.check_symbols(&executor)?;
            let bad_instructions = checker.check_instructions(&executor)?;
            let mut missing_symbols = missing_symbols.wait()?;
            let bad_instructions = bad_instructions.wait()?;
            let mut check_ok = true;
            if !missing_symbols.is_empty() {
                missing_symbols.sort_by(|a, b| {
                    // ignore the leading _Zxxx
                    fn strip(s: &str) -> &str {
                        s.strip_prefix("_Z")
                            .map(|s| s.trim_start_matches(|c: char| c.is_ascii_digit()))
                            .unwrap_or(s)
                    }
                    strip(a).cmp(strip(b))
                });
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
                hintln!(
                    "Hint",
                    "Include the symbols in the linker scripts, or add them to the `ignore` section."
                );
                let missing_symbols = missing_symbols.join("\n");
                let missing_symbols_path = env.target.join("missing_symbols.txt");
                if system::write_file(&missing_symbols_path, &missing_symbols).is_ok() {
                    hintln!(
                        "Saved",
                        "All missing symbols to `{}`",
                        env.from_root(missing_symbols_path).display()
                    );
                } else {
                    hintln!("Error", "Failed to save missing symbols");
                }
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
                let output_path = env.target.join("disallowed_instructions.txt");
                if system::write_file(&output_path, &output).is_ok() {
                    hintln!(
                        "Saved",
                        "All disallowed instructions to {}",
                        env.from_root(output_path).display()
                    );
                } else {
                    hintln!("Error", "Failed to save disallowed instructions");
                }
                check_ok = false;
            }
            if !check_ok {
                errorln!("Error", "Check failed. Please fix the errors above.");
                return Err(report!(Error::Check).attach_printable("Please fix the errors above."));
            }
            hintln!("Checked", "Looks good to me");
        }

        // create the nso after checking
        // so if check failed, nso is not created,
        // and an immediately build with no change won't succeed
        //
        verboseln!("creating nso");

        let mut child = Command::new(&env.elf2nso)
            .args([&env.elf, &env.nso])
            .silent()
            .spawn()
            .change_context(Error::Elf2Nso)?
            .wait()
            .change_context(Error::Elf2Nso)?;
        let result = child.check();
        if result.is_err() {
            child.dump_stderr("Error");
        }
        result.change_context(Error::Elf2Nso)?;

        infoln!("Created", "{}", nso_name);
    }

    if let Some(task) = save_cc_json_task {
        task.wait();
    }

    if let Some(task) = main_npdm_task {
        task.wait()?;
    }

    Ok(format!("{} (profile `{}`)", config.module.name, profile))
}

error_context!(NpdmContext, |r| -> Error { r.change_context(Error::Npdm) });

fn create_npdm(
    target: PathBuf,
    npdmtool: PathBuf,
    title_id: String,
    m_time: FileTime,
) -> ResultIn<(), NpdmContext> {
    verboseln!("creating main.npdm");

    let mut npdm_data: Value = serde_json::from_str(include_str!("template/main.npdm.json"))?;
    npdm_data["title_id"] = json!(format!("0x{}", title_id));
    let npdm_data = serde_json::to_string_pretty(&npdm_data)?;
    let npdm_json = target.join("main.npdm.json");
    system::write_file(&npdm_json, &npdm_data)?;
    system::set_mtime(&npdm_json, m_time)?;
    let main_npdm = target.join("main.npdm");
    // not piping output because it always displays some warning/error
    Command::new(npdmtool)
        .args(args![&npdm_json, &main_npdm])
        .silent()
        .spawn()?
        .wait()?
        .check()?;

    infoln!("Created", "main.npdm");
    Ok(())
}

fn create_verfile(verfile: PathBuf, entry: String) -> Result<(), Error> {
    verboseln!("creating verfile");
    let verfile_data = format!(
        "{}{}{}",
        include_str!("template/verfile.before"),
        entry,
        include_str!("template/verfile.after")
    );
    system::write_file(verfile, &verfile_data).change_context(Error::Verfile)?;
    infoln!("Created", "verfile");
    Ok(())
}

fn symbol_listing_changed(
    config: &Check,
    env: &ProjectEnv,
    m_time: Option<FileTime>,
) -> Result<bool, system::Error> {
    for symbol in &config.symbols {
        let symbol = env.root.join(symbol);
        let symbol_mtime = system::get_mtime(&symbol)?;
        if !system::up_to_date(symbol_mtime, m_time) {
            return Ok(true);
        }
    }
    Ok(false)
}
