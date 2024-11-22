use buildcommon::prelude::*;
use clap::Parser;

use std::io::BufReader;
use std::path::Path;
use std::process::ExitCode;

use buildcommon::env::{Env, RootEnv};
use buildcommon::flags;
use buildcommon::source::{SourceFile, SourceType};
use ninja_writer::*;

#[derive(Debug, Parser)]
struct Cli {
    /// Output file path
    #[clap(short, long)]
    pub output: String,

    /// Patch output of ninja -t compdb (from stdin), and write to output
    #[clap(long)]
    pub compdb: bool,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to load environment")]
    Env,
    #[error("failed to write output file")]
    WriteFile,
    #[error("failed to create build directory")]
    CreateBuildDir,
    #[error("failed while walking source directory")]
    WalkDir,
    #[error("failed to read source directory")]
    ReadDir,
    #[error("failed to read source directory entry")]
    ReadDirEntry,
    #[error("source path is must be utf-8")]
    Encoding,
    #[error("failed to write build.ninja")]
    WriteNinja,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match main_internal(&cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {:?}", e);
            ExitCode::FAILURE
        }
    }
}
fn main_internal(cli: &Cli) -> Result<(), Error> {
    if !cli.compdb {
        build_ninja(cli)?;
        return Ok(());
    }

    let stdin_reader = BufReader::new(std::io::stdin());
    let env = Env::load(None).change_context(Error::Env)?;
    let mut file = system::buf_writer(&cli.output).change_context(Error::WriteFile)?;
    buildcommon::compdb::inject_system_headers(&env, stdin_reader, &mut file)
        .change_context(Error::WriteFile)?;
    Ok(())
}

fn build_ninja(cli: &Cli) -> Result<(), Error> {
    let build_ninja_path = &cli.output;

    let env = RootEnv::from(Env::load(None).change_context(Error::Env)?);
    let lib_root = env.megaton_home.join("lib");
    let src_root = lib_root.join("src");
    let inc_root = lib_root.join("include");

    let build_root = lib_root.join("build").into_joined("bin");
    let build_o_root = build_root.join("o");
    let ninja = Ninja::new();

    let mut build_o_root_str = match build_o_root.as_os_str().to_os_string().into_string() {
        Ok(x) => x,
        Err(_) => {
            return Err(report!(Error::Encoding))
                .attach_printable(format!("path: {}", build_root.join("o").display()))
        }
    };

    system::ensure_path_sep(&mut build_o_root_str);

    system::ensure_directory(&build_o_root).change_context(Error::CreateBuildDir)?;

    ninja.comment("libmegaton build.ninja");
    let common_flags = flags::DEFAULT_COMMON.join(" ");

    ninja.variable("common_flags", &common_flags);

    let includes = [&inc_root, &env.libnx_include];

    // let mut c_flags = flags::DEFAULT_C.to_vec();
    let mut c_flags = vec!["-Wall", "-Werror", "-O3"];
    c_flags.push("-DMEGATON_LIB");

    let include_flag = includes
        .iter()
        .map(|x| format!("-I{}", x.display()))
        .collect::<Vec<_>>()
        .join(" ");

    c_flags.push(&include_flag);

    let mut c_flags = c_flags.join(" ");
    let mut cxx_flags = c_flags.clone();

    c_flags.push_str(" -xc");
    ninja.variable("c_flags", &c_flags);

    cxx_flags.push(' ');
    cxx_flags.push_str(&flags::DEFAULT_CPP.join(" "));
    ninja.variable("cxx_flags", &cxx_flags);

    let as_flags = [format!("-x assembler-with-cpp {}", cxx_flags)];
    let as_flags = as_flags.join(" ");
    ninja.variable("as_flags", &as_flags);
    ninja.variable("cc", &env.cc);
    ninja.variable("cxx", &env.cxx);
    ninja.variable("ar", env.get_dkp_bin("aarch64-none-elf-ar"));

    let rule_as = ninja
        .rule(
            "as",
            "$cxx -MD -MP -MF $out.d $common_flags $as_flags -c $in -o $out",
        )
        .depfile("$out.d")
        .deps_gcc()
        .description("Assembling $out");
    let rule_cc = ninja
        .rule(
            "cc",
            "$cc -MD -MP -MF $out.d $common_flags $c_flags -c $in -o $out",
        )
        .depfile("$out.d")
        .deps_gcc()
        .description("Compiling $out");
    let rule_cxx = ninja
        .rule(
            "cxx",
            "$cxx -MD -MP -MF $out.d $common_flags $cxx_flags -c $in -o $out",
        )
        .depfile("$out.d")
        .deps_gcc()
        .description("Compiling $out");

    let rule_ar = ninja
        .rule("ar", "$ar rcs $out $in")
        .description("Linking $out");

    let mut objects = Vec::new();
    walk_directory(
        &src_root,
        &build_o_root_str,
        &rule_as,
        &rule_cc,
        &rule_cxx,
        &mut objects,
    )
    .change_context(Error::WalkDir)?;

    let libmegaton = build_root.into_joined("libmegaton.a");

    rule_ar.build([&libmegaton]).with(objects);

    let generator = ninja
        .rule(
            "configure",
            "cargo run --quiet --bin megaton-lib-configure -- -o $out",
        )
        .description("Configuring build.ninja")
        .generator();

    generator
        .build([build_ninja_path])
        .with_implicit(["configure.rs"]);

    ninja.defaults([&libmegaton]);

    std::fs::write(build_ninja_path, ninja.to_string()).change_context(Error::WriteNinja)?;

    Ok(())
}

fn walk_directory(
    src: &Path,
    build_o_root: &str,
    rule_as: &RuleRef,
    rule_cc: &RuleRef,
    rule_cxx: &RuleRef,
    objects: &mut Vec<String>,
) -> Result<(), Error> {
    for entry in std::fs::read_dir(src)
        .change_context(Error::ReadDir)
        .attach_printable_lazy(|| format!("reading: {}", src.display()))?
    {
        let entry = entry
            .change_context(Error::ReadDirEntry)
            .attach_printable_lazy(|| format!("inside: {}", src.display()))?;
        let path = entry.path();
        if path.is_dir() {
            walk_directory(&path, build_o_root, rule_as, rule_cc, rule_cxx, objects)?;
            continue;
        }

        let source_file = match SourceFile::from_path(&path).change_context(Error::Encoding)? {
            Some(x) => x,
            None => continue, // not a source type
        };

        let object_file = format!("{}{}.o", build_o_root, source_file.name_hash);
        match source_file.typ {
            SourceType::C => {
                rule_cc.build([&object_file]).with([path]);
            }
            SourceType::Cpp => {
                rule_cxx.build([&object_file]).with([path]);
            }
            SourceType::S => {
                rule_as.build([&object_file]).with([path]);
            }
        }
        objects.push(object_file.clone());
    }

    Ok(())
}
