use std::path::PathBuf;

use cu::pre::*;

pub static MEGATON_TOOLCHAIN_NAME: &str = "megaton";
pub static MEGATON_TOOLCHAIN_SELECTOR: &str = "+megaton";

#[derive(Debug, Serialize)]
pub struct RustEnv {
    pub host: RustHostEnv,
    pub build_dep: RustBuildDepEnv,
    pub megaton: Option<RustToolchainEnv>,
}
impl RustEnv {
    pub fn resolve() -> cu::Result<Self> {
        let host = RustHostEnv::resolve()?;
        let build_dep = RustBuildDepEnv::resolve()?;
        let megaton = RustToolchainEnv::resolve(Some(MEGATON_TOOLCHAIN_NAME))
            .ok();
        Ok(Self { host, build_dep, megaton })
    }
}

#[derive(Debug, Serialize)]
pub struct RustEnvPartial {
    pub host: Option<RustHostEnv>,
    pub build_dep: RustBuildDepEnvPartial,
    pub megaton: Option<RustToolchainEnv>,
}

impl RustEnvPartial {
    pub fn resolve() -> Self {
        let host = RustHostEnv::resolve().ok();
        let build_dep = RustBuildDepEnvPartial::resolve();
        let megaton = RustToolchainEnv::resolve(Some(MEGATON_TOOLCHAIN_NAME))
            .ok();
        Self { host, build_dep, megaton }
    }
}

#[derive(Debug, Serialize)]
pub struct RustHostEnv {
    /// Path to the rustc binary on the host system
    pub rustc: PathBuf,
    /// Path to the rustup binary on the host system
    pub rustup: PathBuf,
    /// Path to the cargo binary on the host system
    pub cargo: PathBuf,
    /// Host target triple
    #[serde(flatten)]
    pub env: RustToolchainEnv,
}

impl RustHostEnv {
    #[cu::context("failed to resolve host rust toolchain")]
    pub fn resolve() -> cu::Result<Self> {
        let rustc = cu::which("rustc")?;
        let rustup = cu::which("rustup")?;
        let cargo = cu::which("cargo")?;
        let env = RustToolchainEnv::resolve(None)?;
        Ok(Self { rustc, rustup, cargo, env })
    }
}

#[derive(Debug, Serialize)]
pub struct RustBuildDepEnv {
    pub git: PathBuf,
    pub ninja: PathBuf,
    pub cmake: PathBuf,
}
impl RustBuildDepEnv {
    #[cu::context("failed to resolve rust toolchain build dependencies")]
    pub fn resolve() -> cu::Result<Self> {
        RustBuildDepEnvPartial::resolve().resolve_full()
    }
    pub fn resolve_partial() -> RustBuildDepEnvPartial {
        RustBuildDepEnvPartial::resolve()
    }
}

#[derive(Debug, Serialize)]
pub struct RustBuildDepEnvPartial {
    pub git: Option<PathBuf>,
    pub ninja: Option<PathBuf>,
    pub cmake: Option<PathBuf>,
}
impl RustBuildDepEnvPartial {
    pub fn resolve() -> Self {
        Self {
            git: cu::which("git").ok(),
            ninja: cu::which("ninja").ok(),
            cmake: cu::which("cmake").ok(),
        }
    }
    pub fn resolve_full(self) -> cu::Result<RustBuildDepEnv> {
        Ok(RustBuildDepEnv {
            git: cu::check!(self.git, "cannot find git, which is required to build the rust toolchain")?,
            ninja: cu::check!(self.ninja, "cannot find ninja, which is required to build llvm for the rust toolchain")?,
            cmake: cu::check!(self.cmake, "cannot find cmake, which is required to build llvm for the rust toolchain")?,
        })
    }
}

/// Information about a rust toolchain obtained from rustc and rustup
#[derive(Debug, Default, Serialize)]
pub struct RustToolchainEnv {
    /// Commit hash
    pub commit_hash: String,
    /// Commit date
    pub commit_date: String,
    /// Host triple this toolchain is built for
    pub host: String,
    /// Release version and channel
    pub release: String,
    /// LLVM version
    pub llvm_version: String,
    /// Targets installed on the toolchain (i.e. what the toolchain
    /// can compile)
    pub targets: Vec<String>,
}

impl RustToolchainEnv {
    /// Get information about a rust toolchain or the default toolchain
    pub fn resolve(toolchain: Option<&str>) -> cu::Result<Self> {
        get_rustc_info(toolchain)
    }
}

/// Get information about the toolchain
#[cu::context("failed to get info for the {} rust toolchain", toolchain.unwrap_or("default"))]
fn get_rustc_info(toolchain: Option<&str>) -> cu::Result<RustToolchainEnv> {
    // rustc +toolchain -vV
    let command = cu::which("rustc")?.command();
    let command = match toolchain {
        Some(toolchain) => command.args([&format!("+{toolchain}"), "-vV"]),
        None => command.arg("-vV"),
    };
    let (child, stdout, stderr) = command
        .stdout(cu::pio::string())
        .stderr(cu::pio::string())
        .stdin_null()
        .spawn()?;
    let result = child.wait_nz();
    let stdout = stdout.join()??;
    let stderr = stderr.join()??;
    cu::check!(result, "{stderr}")?;

    let mut env = RustToolchainEnv::default();
    for line in stdout.lines() {
        if let Some(line) = line.strip_prefix("commit-hash:") {
            env.commit_hash = line.trim().to_string();
            continue;
        }
        if let Some(line) = line.strip_prefix("commit-date:") {
            env.commit_date = line.trim().to_string();
            continue;
        }
        if let Some(line) = line.strip_prefix("host:") {
            env.host = line.trim().to_string();
            continue;
        }
        if let Some(line) = line.strip_prefix("release:") {
            env.release = line.trim().to_string();
            continue;
        }
        if let Some(line) = line.strip_prefix("LLVM version:") {
            env.llvm_version = line.trim().to_string();
            continue;
        }
    }

    // rustup +toolchain target list --installed
    let command = cu::which("rustup")?.command();
    let command = match toolchain {
        Some(toolchain) => command.arg(&format!("+{toolchain}")),
        None => command,
    };
    let (child, stdout, stderr) = command.args(["target", "list", "--installed"])
        .stdout(cu::pio::string())
        .stderr(cu::pio::string())
        .stdin_null()
        .spawn()?;
    let result = child.wait_nz();
    let stdout = stdout.join()??;
    let stderr = stderr.join()??;
    cu::check!(result, "{stderr}")?;

    for line in stdout.lines() {
        env.targets.push(line.trim().to_string());
    }

    Ok(env)
}
