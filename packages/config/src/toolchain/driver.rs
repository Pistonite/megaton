use cu::pre::*;

use crate::toolchain::{CxxBridgeEnv, DevKitA64Env, HomeEnv, RustEnv, RustEnvPartial};

#[derive(Debug, Serialize)]
pub struct ToolchainEnv {
    pub devkita64: DevKitA64Env,
    pub rust: Option<RustEnv>,
    pub home: HomeEnv,
    pub cxxbridge: CxxBridgeEnv,
}

#[derive(Debug, Serialize)]
pub struct ToolchainEnvPartial {
    pub devkita64: Option<DevKitA64Env>,
    pub rust: RustEnvPartial,
    pub home: HomeEnv,
    pub cxxbridge: CxxBridgeEnv,
}

impl ToolchainEnv {
    #[cu::context("failed to resolve toolchain env")]
    pub fn resolve() -> cu::Result<Self> {
        let devkita64 = DevKitA64Env::resolve()?;
        let rust = RustEnv::resolve().inspect_err(|e| cu::debug!("{e}")).ok();
        let home = HomeEnv::resolve()?;
        let cxxbridge = CxxBridgeEnv::resolve(&home);

        Ok(Self {
            devkita64,
            rust,
            home, cxxbridge
        })
    }

    pub fn resolve_partial() -> cu::Result<ToolchainEnvPartial> {
        let devkita64 = DevKitA64Env::resolve().ok();
        let rust = RustEnvPartial::resolve();
        let home = HomeEnv::resolve()?;
        let cxxbridge = CxxBridgeEnv::resolve(&home);

        Ok(ToolchainEnvPartial {
            devkita64,
            rust,
            home, cxxbridge
        })
    }

    /// Turn the toolchain env into a JSON blob
    pub fn to_json(&self) -> cu::Result<json::Value> {
        json::to_value(self)
    }
}
impl ToolchainEnvPartial {
    /// Turn the toolchain env into a JSON blob
    pub fn to_json(&self) -> cu::Result<json::Value> {
        json::to_value(self)
    }
}
