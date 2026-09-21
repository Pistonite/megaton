use std::path::{Path, PathBuf};

use cu::pre::*;

static TRIPLE: &str = "aarch64-none-elf";

#[derive(Debug, Serialize)]
pub struct DevKitA64Env {
    /// The GCC version DKA64 is based on
    pub version: String,
    /// The C compiler path
    #[serde(rename = "CC")]
    pub cc: String,
    /// The CXX compiler path
    #[serde(rename = "CXX")]
    pub cxx: String,
    /// The AS assembler path
    #[serde(rename = "AS")]
    pub asm: String,
    /// Archiver path
    #[serde(rename = "AR")]
    pub ar: String,
    /// Objdump path
    #[serde(rename = "OBJDUMP")]
    pub objdump: String,
    /// Npdmtool path
    #[serde(rename = "NPDMTOOL")]
    pub npdmtool: String,
    /// Elf2nso tool path
    #[serde(rename = "ELF2NSO")]
    pub elf2nso: String,
    /// System include paths for C
    pub c_includes: Vec<String>,
    /// System include paths for CXX
    pub cpp_includes: Vec<String>,
}

impl DevKitA64Env {
    /// Resolve DKA64 environment from the DEVKITPRO environment variable.
    ///
    /// Note we need other tools that are not part of DKA64 but part
    /// of the shared tools so just having the A64 toolchain is not enough
    #[cu::context("failed to resolve DevKitA64 toolchain")]
    pub fn resolve() -> cu::Result<Self> {
        let dkp = cu::env_var("DEVKITPRO").unwrap_or_default();
        let (dkp, ok) = if dkp.is_empty() {
            cu::hint!("DEVKITPRO environment variable is not set!");
            (PathBuf::new(), false)
        } else {
            let path = PathBuf::from(dkp).normalize_exists();
            match path {
                Ok(p) => {
                    if !p.is_dir() {
                        cu::hint!("DEVKITPRO environment variable is not set to a valid directory.");
                        (p, false)
                    } else {
                        (p, true)
                    }
                }
                Err(_) => {
                    cu::hint!("DEVKITPRO environment variable is not set to a valid path.");
                    (Default::default(),false)
                }
            }
        };
        if !ok {
            cu::hint!("please ensure DevKitA64 is installed and set the DEVKITPRO environment variable");
            cu::bail!("failed to resolve DEVKITPRO");
        }
        cu::check!(dkp.as_utf8(), "DEVKITPRO path must be utf-8")?;
        let tools_bin = cu::path!(&dkp / "tools" / "bin");
        let dka64 = cu::path!(dkp / "devkitA64");
        let dka64_bin = cu::path!(&dka64 / "bin");
        let cc = cu::path!(&dka64_bin / format!("{TRIPLE}-gcc"));

        let version = match get_gcc_version_from_installation(&dka64) {
            Some(v) => {
                cu::debug!("detected gcc version from installation: {v}");
                v
            }
            None => {
                cu::debug!("invoking gcc to check its version");
                let v = get_gcc_version_from_gcc(&cc)?;
                cu::debug!("detected gcc version from gcc: {v}");
                v
            }
        };

        let cxx = cu::path!(&dka64_bin / format!("{TRIPLE}-g++"));
        let asm = cu::path!(&dka64_bin / format!("{TRIPLE}-as"));
        let ar = cu::path!(&dka64_bin / format!("{TRIPLE}-ar"));
        let objdump = cu::path!(&dka64_bin / format!("{TRIPLE}-objdump"));
        let npdmtool = cu::path!(&tools_bin / "npdmtool");
        let elf2nso = cu::path!(&tools_bin / "elf2nso");

        let (c_includes, cpp_includes) = get_includes(&dka64, &version)?;

        Ok(Self {
            version,
            cc: cc.into_utf8()?,
            cxx: cxx.into_utf8()?,
            asm: asm.into_utf8()?,
            ar: ar.into_utf8()?,
            objdump: objdump.into_utf8()?,
            npdmtool: npdmtool.into_utf8()?,
            elf2nso: elf2nso.into_utf8()?,
            c_includes,
            cpp_includes,
        })

    }
}

fn get_gcc_version_from_installation(dka64: &Path) -> Option<String> {
    let dka64_include_cpp = cu::path!(&dka64 / TRIPLE / "include" / "c++");
    // the include directory should have 1 version
    let readdir = match cu::fs::read_dir(&dka64_include_cpp) {
        Ok(readdir) => readdir,
        Err(e) => {
            cu::debug!("failed to read c++ include path: {e:?}");
            return None;
        }
    };
    let mut candidate = None;
    for entry in readdir {
        let entry = match entry {
            Err(e) => {
                cu::debug!("error reading some entry from devkitpro c++ include path: {e:?}");
                continue;
            }
            Ok(x) => x
        };
        let version = match entry.file_name().into_utf8() {
            Err(e) => {
                cu::debug!("ignoring non-utf8 while reading devkitpro c++ version: {e:?}");
                continue;
            }
            Ok(x) => x
        };
        match candidate {
            None => {
                candidate = Some(version);
            }
            Some(old_version) => {
                cu::debug!("more than one version is found ({version} and {old_version}), requires calling gcc to resolve");
                return None;
            }
        }
    }

    if candidate.is_none() {
        cu::debug!("no gcc versions found in c++ include path");
    }

    candidate
}

#[cu::context("failed to parse gcc version (path: '{}')", cc_path.display())]
fn get_gcc_version_from_gcc(cc_path: &Path) -> cu::Result<String> {
    let (child, _, output) = cc_path
        .command()
        .arg("-v")
        .stdio_null()
        .stderr(cu::pio::string())
        .spawn()?;
    child.wait_nz()?;
    let output = output.join()??;
    let verline = output.lines().last().unwrap_or(&output);
    let Some(verstring) = verline.split(" ").nth(2) else {
        cu::error!("cannot determine version from gcc output:\n{output}");
        cu::bail!("cannot determine gcc version: failed to parse output");
    };

    Ok(verstring.to_owned())
}

fn get_includes(dka64: &Path, version: &str) -> cu::Result<(Vec<String>, Vec<String>)> {
    let dka64_triple_include = cu::path!(&dka64 / TRIPLE / "include");
    let dka64_triple_include_cpp = cu::path!(&dka64_triple_include / "c++" / version);
    let dka64_lib_gcc_include = cu::path!(&dka64 / "lib" / "gcc" / TRIPLE / version);

    Ok((vec![
        dka64_triple_include.into_utf8()?,
        dka64_lib_gcc_include.join("include").into_utf8()?,
        cu::path!(dka64_lib_gcc_include / "include-fixed").into_utf8()?
    ], vec![
        dka64_triple_include_cpp.join(TRIPLE).into_utf8()?,
        dka64_triple_include_cpp.join("backward").into_utf8()?,
        dka64_triple_include_cpp.into_utf8()?,
    ]))
}
