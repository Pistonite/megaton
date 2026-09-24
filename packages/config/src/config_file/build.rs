use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::config_file::{self, CaptureUnused, ExtendProfile, ProjectTargetEnv, Resolve, Validate, ValidateCtx};
use crate::toolchain::{DEVKITA64_TRIPLE, ToolchainEnv};

/// Config in the `[build]` section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildConfig {
    /// If STD support is enabled.
    ///
    /// When false, megaton just links rocrt for you. You need
    /// to provide megaton_nnmain
    #[serde(default = "BuildConfig::default_std")]
    pub std: bool,

    /// C++ STD version, should be a string passed to -std
    #[serde(rename = "std-c++", default = "BuildConfig::default_std_cpp")]
    pub std_cpp: String,
    #[serde(rename = "std-c", default = "BuildConfig::default_std_c")]
    pub std_c: String,

    /// C/C++ Source directories
    #[serde(default)]
    pub sources: Vec<PathBuf>,

    /// C/C++ Include directories
    #[serde(default)]
    pub includes: Vec<PathBuf>,

    /// C/C++ System Include directories (-isystem)
    #[serde(default)]
    pub system_includes: Vec<PathBuf>,

    /// If defined, passed in as --sysroot flag
    pub sysroot: Option<PathBuf>,

    /// C/C++ Defines in the form of `K=V`, the string is treated
    /// literally by prepending -D
    #[serde(default)]
    pub defines: Vec<String>,

    /// Additional Linker scripts
    #[serde(default)]
    pub ldscripts: Vec<PathBuf>,

    /// Additional objects to link
    ///
    /// These are linked directly after the compiled objects, right before
    /// linking libraries
    #[serde(default)]
    pub objects: Vec<PathBuf>, // TODO - maybe we need an object graph?

    #[serde(default)]
    pub compiler: BuildCompilerConfig,

    #[serde(default)]
    pub flags: BuildFlagConfig,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}
impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            std: Self::default_std(),
            std_cpp: Self::default_std_cpp(),
            std_c: Self::default_std_c(),
            sources: Default::default(),
            includes: Default::default(),
            system_includes: Default::default(),
            sysroot: Default::default(),
            defines: Default::default(),
            ldscripts: Default::default(),
            objects: Default::default(),
            compiler: Default::default(),
            flags: Default::default(),
            unused: Default::default(),
        }
    }
}
impl BuildConfig {
    fn default_std() -> bool {
        true
    }
    fn default_std_cpp() -> String {
        "c++23".to_string()
    }
    fn default_std_c() -> String {
        "c23".to_string()
    }
}

impl Resolve for BuildConfig {
    fn resolve(&mut self, project: &ProjectTargetEnv, toolchain: Option<&ToolchainEnv>) -> cu::Result<()> {
        for p in &mut self.sources {
            *p = project.root.join(&*p);
        }
        for p in &mut self.includes {
            *p = project.root.join(&*p);
        }
        for p in &mut self.system_includes {
            *p = project.root.join(&*p);
        }
        for p in &mut self.ldscripts {
            *p = project.root.join(&*p);
        }
        for p in &mut self.objects {
            *p = project.root.join(&*p);
        }
        if let Some(p) = &mut self.sysroot {
            *p = project.root.join(&*p);
        }
        self.compiler.resolve(project, toolchain)?;
        let mut flags = std::mem::take(&mut self.flags);
        flags.resolve(project, &self, toolchain)?;
        self.flags = flags;
        Ok(())
    }
}

impl Validate for BuildConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.flags.validate_property(ctx, "flags")?;
        self.compiler.validate_property(ctx, "compiler")?;
        self.unused.validate(ctx)?;

        if self.std {
            if !self.compiler.cc_flavor.is_dkp() || !self.compiler.cxx_flavor.is_dkp() {
                cu::bail!("DevKitA64 toolchain must be used when std is enabled; either remove build.compiler.C and build.compiler.CXX to use DevKitA64, or set build.std = false to disable standard library support.");
            }
        }
        Ok(())
    }
}

impl ExtendProfile for BuildConfig {
    fn extend_profile(&mut self, other: &Self) {
        config_file::extend_by_appending(&mut self.sources, &other.sources);
        config_file::extend_by_appending(&mut self.includes, &other.includes);
        config_file::extend_by_appending(&mut self.system_includes, &other.system_includes);
        config_file::extend_by_overriding_if_some(&mut self.sysroot, other.sysroot.as_ref());
        config_file::extend_by_appending(&mut self.defines, &other.defines);
        config_file::extend_by_appending(&mut self.ldscripts, &other.ldscripts);
        config_file::extend_by_appending(&mut self.objects, &other.objects);

        self.compiler.extend_profile(&other.compiler);
        self.flags.extend_profile(&other.flags);
    }
}

/// `[build.compiler]` config section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildCompilerConfig {
    /// Override the C compiler. This cannot be used to supply extra arguments
    ///
    /// Clang is supported if the compiler name contains `clang`. The default flags
    /// will be converted to the clang equivalents
    #[serde(rename = "CC")]
    cc: Option<String>,
    #[serde(skip_deserializing)]
    pub cc_flavor: CompilerFlavor,
    /// Override the CXX compiler. This cannot be used to supply extra arguments
    ///
    /// Clang is supported if the compiler name contains `clang`. The default flags
    /// will be converted to the clang equivalents
    #[serde(rename = "CXX")]
    cxx: Option<String>,
    #[serde(skip_deserializing)]
    pub cxx_flavor: CompilerFlavor,

    /// Override the AS assembler. Must be GNU Assembler.
    ///
    /// This doesn't actually change how assembly files are compiled, since
    /// the GCC driver is used for assembling. But this is passed to cargo invocation.
    #[serde(rename = "AS")]
    asm: Option<String>,
    // pub ar: Option<Vec<String>>,
    /// Override CXX compiler for linking
    #[serde(rename = "CXXLD")]
    cxx_ld: Option<String>,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}
impl BuildCompilerConfig {
    #[cu::context("failed to resolve build.compiler")]
    pub fn resolve(&mut self, project: &ProjectTargetEnv, toolchain: Option<&ToolchainEnv>) -> cu::Result<()> {
        match &self.cc {
            None => {
                self.cc = match toolchain {
                    None => Some(format!("{DEVKITA64_TRIPLE}-gcc")),
                    Some(tc) => Some(tc.devkita64.cc.clone())
                };
                self.cc_flavor = CompilerFlavor::Dkp;
            }
            Some(x) => {
                self.cc = Some(Self::resolve_program(&project.root, x, "C compiler")?);
                let cc_is_clang = self.cc.as_deref().unwrap_or_default().contains("clang");
                self.cc_flavor = if cc_is_clang { CompilerFlavor::Clang } else {
                    CompilerFlavor::Gcc
                };
            }
        }
        match &self.cxx {
            None => {
                self.cxx = match toolchain {
                    None => Some(format!("{DEVKITA64_TRIPLE}-g++")),
                    Some(tc) => Some(tc.devkita64.cxx.clone())
                };
                self.cxx_flavor = CompilerFlavor::Dkp;
            }
            Some(x) => {
                self.cxx = Some(Self::resolve_program(&project.root, x, "CXX compiler")?);
                let cxx_is_clang = self.cxx.as_deref().unwrap_or_default().contains("clang");
                self.cxx_flavor = if cxx_is_clang { CompilerFlavor::Clang } else {
                    CompilerFlavor::Gcc
                };
            }
        }
        match &self.asm {
            None => {
                self.asm = match toolchain {
                    None => Some(format!("{DEVKITA64_TRIPLE}-as")),
                    Some(tc) => Some(tc.devkita64.asm.clone())
                };
            }
            Some(x) => {
                self.asm = Some(Self::resolve_program(&project.root, x, "AS assembler")?);
            }
        }
        match &self.cxx_ld {
            None => {
                self.cxx_ld = match toolchain {
                    None => Some(format!("{DEVKITA64_TRIPLE}-g++")),
                    Some(tc) => Some(tc.devkita64.cxx.clone())
                };
            }
            Some(x) => {
                self.cxx_ld = Some(Self::resolve_program(&project.root, x, "CXX compiler for linking")?);
            }
        }

        Ok(())
    }
    fn resolve_program(root: &Path, program: &str, desc: &str) -> cu::Result<String> {
        let p = if program.starts_with('.') {
            // this should be utf-8 since root should be, and program is str
            cu::which(root.join(program).as_utf8()?)
        } else {
            cu::which(&program)
        };
        let x = cu::check!(p, "failed to find {desc}")?;
        let x = cu::check!(x.into_utf8(), "path to {desc} is not utf-8")?;
        Ok(x)
    }
    pub fn cc(&self) -> &str {
        self.cc.as_deref().unwrap_or_default()
    }
    pub fn cxx(&self) -> &str {
        self.cxx.as_deref().unwrap_or_default()
    }
    pub fn asm(&self) -> &str {
        self.asm.as_deref().unwrap_or_default()
    }
    pub fn cxx_ld(&self) -> &str {
        self.cxx_ld.as_deref().unwrap_or_default()
    }
}
impl Validate for BuildCompilerConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}
impl ExtendProfile for BuildCompilerConfig {
    fn extend_profile(&mut self, other: &Self) {
        config_file::extend_by_overriding_if_some(&mut self.cc, other.cc.as_ref());
        config_file::extend_by_overriding_if_some(&mut self.cxx, other.cxx.as_ref());
        config_file::extend_by_overriding_if_some(&mut self.asm, other.asm.as_ref());
        config_file::extend_by_overriding_if_some(&mut self.cxx_ld, other.cxx_ld.as_ref());
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompilerFlavor {
    #[default]
    Dkp,
    Gcc,
    Clang,
}
impl CompilerFlavor {
    pub fn is_dkp(self) -> bool {
        matches!(self, Self::Dkp)
    }
    pub fn is_gcc(self) -> bool {
        matches!(self, Self::Gcc | Self::Dkp)
    }
    pub fn is_non_dkp_gcc(self) -> bool {
        matches!(self, Self::Gcc)
    }
    pub fn is_clang(self) -> bool {
        matches!(self, Self::Clang)
    }
}

/// `[build.flags]` config section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildFlagConfig {
    /// Common flags
    common: Option<Vec<String>>,
    /// C flags (extends common flags)
    c: Option<Vec<String>>,
    /// CXX flags (extends C flags)
    cxx: Option<Vec<String>>,
    /// AS flags passed from GCC driver (extends common flags)
    cc_asm: Option<Vec<String>>,
    /// CXXLD flags (extends common flags)
    cxx_ld: Option<Vec<String>>,
    /// RUSTFLAGS
    rust: Option<Vec<String>>,
    /// Cargo flags, such as feature flags
    cargo: Option<Vec<String>>,

    #[serde(skip_deserializing)]
    resolved: BuildFlags,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}

impl BuildFlagConfig {
    #[cu::context("failed to resolve build.flags")]
    pub fn resolve(&mut self, project: &ProjectTargetEnv, build: &BuildConfig, toolchain: Option<&ToolchainEnv>) -> cu::Result<()> {
        let common = config_file::resolve_default_token(
            vec![],
            self.common.as_ref().map(|x| x.as_slice()),
            || {
                DEFAULT_COMMON
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
            },
        );
        let mut c = config_file::resolve_default_token(
            vec![],
            self.c.as_ref().map(|x| x.as_slice()),
            || {
                let mut out = common.clone();
                out.extend(DEFAULT_C_COMMON.iter().map(|x| x.to_string()));
                if build.compiler.cc_flavor.is_clang() {
                    out.extend(DEFAULT_C_CLANG.iter().map(|x| x.to_string()));
                }
                out
            }
        );
        let mut cxx = config_file::resolve_default_token(
            vec![],
            self.cxx.as_ref().map(|x| x.as_slice()),
            || {
                let mut out = c.clone();
                out.extend(DEFAULT_CPP_COMMON.iter().map(|x| x.to_string()));
                if build.compiler.cxx_flavor.is_clang() {
                    out.extend(DEFAULT_CPP_CLANG.iter().map(|x| x.to_string()));
                }
                out
            }
        );
        c.push(format!("-std={}", build.std_c));
        cxx.push(format!("-std={}", build.std_cpp));
        let mut cc_asm = config_file::resolve_default_token(
            vec![],
            self.cc_asm.as_ref().map(|x| x.as_slice()),
            || {
                let mut out = common.clone();
                out.extend(DEFAULT_ASM_GCC.iter().map(|x| x.to_string()));
                out
            }
        );
        let mut cxx_ld = config_file::resolve_default_token(
            vec![],
            self.cxx_ld.as_ref().map(|x| x.as_slice()),
            || {
                let mut out = common.clone();
                out.extend(DEFAULT_LINKER.iter().map(|x| x.to_string()));
                out
            }
        );
        let rustflags = config_file::resolve_default_token(
            vec![],
            self.rust.as_ref().map(|x| x.as_slice()),
            || {
                DEFAULT_RUST.iter().map(|x| x.to_string()).collect()
            }
        );
        let cargo = config_file::resolve_default_token(
            vec![],
            self.cargo.as_ref().map(|x| x.as_slice()),
            || {
                DEFAULT_CARGO.iter().map(|x| x.to_string()).collect()
            }
        );

        // defines
        for define in &build.defines {
            let flag = format!("-D{define}");
            for flagset in [&mut c, &mut cxx, &mut cc_asm] {
                flagset.push(flag.clone());
            }
        }

        fn make_flag(prefix: &str, p: &str) -> cu::Result<String> {
            if p.contains('"') {
                cu::bail!("path cannot contain quotes; paths are quoted automatically if needed when converted to flags: '{p}'");
            }
            let flag = if p.contains(' ') {
                format!("{prefix}\"{p}\"")
            } else {
                format!("{prefix}{p}")
            };
            Ok(flag)
        }

        // includes
        // - user includes first (the ones in Megaton.toml)
        // - then megaton library ones
        // - then user system-includes
        // - dkp ones are only added to compdb if compiler is dkp
        for include in &build.includes {
            let include_str = cu::check!(include.as_utf8(), "(in build.includes) include path must be UTF-8")?;
            let flag = make_flag("-I", include_str)?;
            for flagset in [&mut c, &mut cxx, &mut cc_asm] {
                flagset.push(flag.clone());
            }
        }
        // TODO: figure out the megaton library directories
        for mega_library in ["mega-libcpp"] {
            let include_path = cu::path!(&(&project.lib_root) / mega_library / "include");
            let include_str = cu::check!(include_path.as_utf8(), "megaton include path must be UTF-8; please put the project in another location")?;
            let flag = make_flag("-I", include_str)?;
            for flagset in [&mut c, &mut cxx, &mut cc_asm] {
                flagset.push(flag.clone());
            }
        }
        for include in &build.system_includes {
            let include_str = cu::check!(include.as_utf8(), "(in build.system-includes) system include path must be UTF-8")?;
            let flag = make_flag("-isystem", include_str)?;
            for flagset in [&mut c, &mut cxx, &mut cc_asm] {
                flagset.push(flag.clone());
            }
        }

        let mut compdb_c = c.clone();
        let mut compdb_cxx = cxx.clone();
        let mut compdb_cc_asm = cc_asm.clone();
        if build.compiler.cc_flavor.is_dkp() {
            if let Some(tc) = toolchain {
                for include in &tc.devkita64.c_includes {
                    let flag = make_flag("-isystem", include)?;
                    compdb_c.push(flag.clone());
                    compdb_cc_asm.push(flag);
                }
            }
        }
        if build.compiler.cxx_flavor.is_dkp() {
            if let Some(tc) = toolchain {
                for include in &tc.devkita64.c_includes {
                    let flag = make_flag("-isystem", include)?;
                    compdb_cxx.push(flag.clone());
                }
                for include in &tc.devkita64.cpp_includes {
                    let flag = make_flag("-isystem", include)?;
                    compdb_cxx.push(flag.clone());
                }
            }
        }

        // linker
        // - version script
        // - linker scripts
        // - library order
        let verfile = cu::path!(&(&project.lib_root) / "mega-libcpp" / "linker" / "verfile");
        cxx_ld.push(make_flag("-Wl,--version-script=", verfile.as_utf8()?)?);
        let megaton_linker_script = cu::path!(&(&project.lib_root) / "mega-libcpp" / "linker" / "megaton.ld");
        cxx_ld.push(make_flag("-Wl,-T,", megaton_linker_script.as_utf8()?)?);
        for linker_script in &build.ldscripts {
            let path_str = cu::check!(linker_script.as_utf8(), "linker script path must be UTF-8")?;
            cxx_ld.push(make_flag("-Wl,-T,", path_str)?);
        }
        for object in &build.objects {
            let path_str = cu::check!(object.as_utf8(), "object path must be UTF-8")?;
            cxx_ld.push(make_flag("", path_str)?);
        }
        let mut cxx_ld_after_obj = vec![];
        // TODO: figure out what megaton objects/libraries as well as system libraries need to be linked
        let lib = cu::path!(&(&project.lib_root) / "mega-libcpp" / "lib" / "rocrt0.o");
        cxx_ld_after_obj.push(make_flag("", lib.as_utf8()?)?);
        let lib = cu::path!(&(&project.lib_root) / "mega-libcpp" / "lib" / "rocrt1.o");
        cxx_ld_after_obj.push(make_flag("", lib.as_utf8()?)?);
        let lib = cu::path!(&(&project.lib_root) / "mega-libcpp" / "lib" / "libmegatoncpp.a");
        cxx_ld_after_obj.push(make_flag("", lib.as_utf8()?)?);

        cxx_ld_after_obj.push(make_flag("-o", project.out_elf.as_utf8()?)?);
        
        self.resolved.c = c;
        self.resolved.cxx = cxx;
        self.resolved.cc_asm = cc_asm;
        self.resolved.cxx_ld = cxx_ld;
        self.resolved.cxx_ld_after_obj = cxx_ld_after_obj;
        self.resolved.rustflags = rustflags.join(" ");
        self.resolved.cargo = cargo;
        self.resolved.compdb.c = compdb_c;
        self.resolved.compdb.cxx = compdb_cxx;
        self.resolved.compdb.cc_asm = compdb_cc_asm;

        Ok(())
    }
}

impl ExtendProfile for BuildFlagConfig {
    fn extend_profile(&mut self, other: &Self) {
        config_file::extend_by_default_token(&mut self.common, other.common.as_ref());
        config_file::extend_by_default_token(&mut self.c, other.c.as_ref());
        config_file::extend_by_default_token(&mut self.cxx, other.cxx.as_ref());
        config_file::extend_by_default_token(&mut self.cc_asm, other.cc_asm.as_ref());
        config_file::extend_by_default_token(&mut self.cxx_ld, other.cxx_ld.as_ref());
        config_file::extend_by_default_token(&mut self.rust, other.rust.as_ref());
        config_file::extend_by_default_token(&mut self.cargo, other.cargo.as_ref());
    }
}
impl Validate for BuildFlagConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}

/// Resolved build flags.
///
/// These don't yet include:
/// - Input/Output flags like -MF, -c, -o
/// - Objects passed to the linker
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildFlags {
    /// Resolved C flags passed to the compiler
    pub c: Vec<String>,
    /// Resolved CXX flags passed to the compiler
    pub cxx: Vec<String>,
    /// Resolved flags passed to the GCC driver for assembling
    pub cc_asm: Vec<String>,
    /// Resolved linker flags
    pub cxx_ld: Vec<String>,
    /// Resolved linker flags after objects (i.e. archives and -l libraries)
    pub cxx_ld_after_obj: Vec<String>,
    /// Resolved RUSTFLAGS
    pub rustflags: String,
    /// Resolved Cargo flags
    pub cargo: Vec<String>,
    /// Flags for in compile_commands.json for compatibility with clangd (needs explicit system
    /// headers)
    pub compdb: CompdbFlags,
}

/// For compatibility with compile_commands.json
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CompdbFlags {
    /// Resolved C flags for compatibility with compile_commands.json
    pub c: Vec<String>,
    /// Resolved CXX flags for compatibility with compile_commands.json
    pub cxx: Vec<String>,
    /// Resolved flags passed to the GCC driver for compatibility with compile_commands.json
    pub cc_asm: Vec<String>,
}

/// Default common flags, works for both GCC/Clang
static DEFAULT_COMMON: &[&str] = &[
    // arch/tune works for both gcc and clang
    "-march=armv8-a+crc+crypto",
    "-mtune=cortex-a57",
    // this cannot be enabled if we want real TLS
    // "-mtp=soft",

    // position-independent code, needed for TLS relocation
    "-fPIC",
    // needed for gc-sections
    "-fvisibility=hidden",
    // debug info
    "-g",
];

/// Default C flags for GCC and Clang
static DEFAULT_C_COMMON: &[&str] = &[
    // strict (can remove individual with -Wno-...)
    "-Wall",
    "-Wextra",
    // for gc-sections later
    "-ffunction-sections",
    "-fdata-sections",
    // needed to make sure functions in headers are inlined
    "-O3",
    // for exception handling
    "-funwind-tables",
    "-fasynchronous-unwind-tables",
];

/// Default C flags for Clang only (appended to shared ones)
static DEFAULT_C_CLANG: &[&str] = &["-fno-emulated-tls", "--target=aarch64-none-elf"];

/// Default AS flags
static DEFAULT_ASM_GCC: &[&str] = &[];

/// Default CXX flags
static DEFAULT_CPP_COMMON: &[&str] = &["-fno-rtti"];

/// Default CXX flags for Clang (appended to shared ones)
static DEFAULT_CPP_CLANG: &[&str] = &["-nostdinc++"];

/// Default CXX flags when linking
static DEFAULT_LINKER: &[&str] = &[
    "-shared",
    // all startfiles are done by us in rocrt
    "-nostartfiles",
    // we link the std libs ourselves since order matters
    "-nodefaultlibs",
    // "-Wl,-pie", ? [ note we have -fPIC, apparently this must not be added? ]
    "-Wl,--shared", // should be redundant, include anyway
    // disallow relocation into read only pages
    "-Wl,-z,text",
    "-Wl,-z,nodynamic-undefined-weak",
    "-Wl,--build-id=sha1",
    // include eh_frame_hdr for exceptions
    "-Wl,–eh-frame-hdr",
    // do not export symbols from archives
    "-Wl,–exclude-libs,ALL",
    "-Wl,--gc-sections",
    "-Wl,--nx-module-name",
    // _init/_fini are the defaults, (called by nnrtld)
    // include anyway so it's not black magic
    "-Wl,-init=_init",
    "-Wl,-fini=_fini",
];

pub static DEFAULT_RUST: &[&str] = &[];
pub static DEFAULT_CARGO: &[&str] = &[
    "--release",
    // set the target
    "--target",
    "aarch64-unknown-hermit",
];
