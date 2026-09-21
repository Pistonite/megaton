use cu::pre::*;

use crate::toolchain::DevKitA64Env;

#[derive(Debug, Serialize)]
pub struct ToolchainEnv {
    pub devkita64: DevKitA64Env
}

impl ToolchainEnv {
    #[cu::context("failed to resolve toolchain env")]
    pub fn resolve() -> cu::Result<Self> {
        Ok(Self {
            devkita64: DevKitA64Env::resolve()?
        })
    }
}
