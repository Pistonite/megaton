use buildcommon::{flags, prelude::*};

use std::ffi::OsStr;
use std::path::Path;
use std::process::ExitCode;

use buildcommon::env::{Env, RootEnv};
use ninja_writer::*;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("invalid argument. First arg should be path to build.ninja")]
    Arg,
    #[error("failed to load environment")]
    Env,
    #[error("failed to create build directory")]
    CreateBuildDir,
    #[error("failed while walking source directory")]
    WalkDir,
    #[error("failed to read source directory")]
    ReadDir,
    #[error("failed to read source directory entry")]
    ReadDirEntry,
    #[error("failed to get source file or directory name")]
    SourcePathName,
    #[error("source path is must be utf-8")]
    Encoding,
    #[error("failed to write build.ninja")]
    WriteNinja,
}

fn main() -> ExitCode {
    match main_internal() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {:?}", e);
            ExitCode::FAILURE
        }
    }
}

fn main_internal() -> Result<(), Error> {
    let mut args = std::env::args().skip(1);
    let build_ninja_path = args.next().ok_or(Error::Arg)?;

    let env = RootEnv::from(Env::load(None).change_context(Error::Env)?);
    let lib_root = env.megaton_home.join("lib");
    let src_root = lib_root.join("src");
    let inc_root = lib_root.join("include");

    let build_root = lib_root.join("build").into_joined("out");
    let ninja = Ninja::new();

    system::ensure_directory(&build_root).change_context(Error::CreateBuildDir)?;

    ninja.comment("libmegaton build.ninja");
    let common_flags = flags::DEFAULT_COMMON.join(" ");

    ninja.variable("common_flags", &common_flags);

    let exl_inc_root = src_root.join("exlaunch").into_joined("include");

    let includes = [&inc_root, &exl_inc_root, &env.libnx_include];

    let mut c_flags = flags::DEFAULT_C.to_vec();
    // temp. no longer needed when EXL is refactored
    c_flags.extend([
        "-DEXL_DEBUG",
        "-DEXL_USE_FAKEHEAP",
        "-DEXL_LOAD_KIND_ENUM=2",
        "-DEXL_LOAD_KIND=Module",
        "-DEXL_PROGRAM_ID=0x0100000000000000",
        "-DEXL_MODULE_NAME='\"test\"'",
    ]);

    let include_flag = includes
        .iter()
        .map(|x| format!("-I{}", x.display()))
        .collect::<Vec<_>>()
        .join(" ");

    c_flags.push(&include_flag);

    let c_flags = c_flags.join(" ");
    ninja.variable("c_flags", &c_flags);

    let cxx_flags = flags::DEFAULT_CPP.join(" ");
    ninja.variable("cxx_flags", &cxx_flags);

    let as_flags = [format!("-x assembler-with-cpp {}", cxx_flags)];
    let as_flags = as_flags.join(" ");
    ninja.variable("as_flags", &as_flags);
    ninja.variable("cc", &env.cc);
    ninja.variable("cxx", &env.cxx);

    let rule_as = ninja
        .rule(
            "as",
            "$cc -MD -MP -MF $out.d $as_flags $common_flags -c $in -o $out",
        )
        .depfile("$out.d")
        .deps_gcc()
        .description("AS $out");
    let rule_cc = ninja
        .rule(
            "cc",
            "$cc -MD -MP -MF $out.d $common_flags $c_flags -c $in -o $out",
        )
        .depfile("$out.d")
        .deps_gcc()
        .description("CC $out");
    let rule_cxx = ninja
        .rule(
            "cxx",
            "$cxx -MD -MP -MF $out.d $common_flags $c_flags $cxx_flags -c $in -o $out",
        )
        .depfile("$out.d")
        .deps_gcc()
        .description("CXX $out");
    walk_directory(&src_root, &build_root, &rule_as, &rule_cc, &rule_cxx)
        .change_context(Error::WalkDir)?;

    let generator = ninja
        .rule(
            "configure",
            "cargo --color always run --bin megaton-lib-configure -- $out",
        )
        .description("Configuring build.ninja")
        .generator();

    generator
        .build([&build_ninja_path])
        .with_implicit(["configure.rs"]);

    std::fs::write(build_ninja_path, ninja.to_string()).change_context(Error::WriteNinja)?;

    Ok(())
}

fn walk_directory(
    src: &Path,
    build: &Path,
    rule_as: &RuleRef,
    rule_cc: &RuleRef,
    rule_cxx: &RuleRef,
) -> Result<(), Error> {
    let mut create_build = false;

    for entry in std::fs::read_dir(src)
        .change_context(Error::ReadDir)
        .attach_printable_lazy(|| format!("reading: {}", src.display()))?
    {
        let entry = entry
            .change_context(Error::ReadDirEntry)
            .attach_printable_lazy(|| format!("inside: {}", src.display()))?;
        let path = entry.path();
        let name = path.file_name().ok_or(Error::SourcePathName)?;
        if path.is_dir() {
            walk_directory(&path, &build.join(name), rule_as, rule_cc, rule_cxx)?;
        } else {
            if let Some(ext) = path.extension() {
                let out_name = format!(
                    "{}.o",
                    name.to_str()
                        .ok_or(Error::Encoding)
                        .attach_printable_lazy(|| format!("path: {}", path.display()))?
                );
                let build_path = build.join(out_name);

                if ext == OsStr::new("s") {
                    rule_as.build([build_path]).with([path]);
                    create_build = true;
                } else if ext == OsStr::new("c") {
                    rule_cc.build([build_path]).with([path]);
                    create_build = true;
                } else if ext == OsStr::new("cpp") {
                    rule_cxx.build([build_path]).with([path]);
                    create_build = true;
                }
            }
        }
    }
    if create_build && !build.exists() {
        std::fs::create_dir_all(build)?;
    }

    Ok(())
}
