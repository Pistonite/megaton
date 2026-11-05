use std::path::PathBuf;

use cu::pre::*;

use crate::config_file::{CaptureUnused, ExtendProfile, ProjectTargetEnv, Resolve, Validate, ValidateCtx};
use crate::toolchain::ToolchainEnv;

/// The `check` section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CheckConfig {
    /// Symbols to ignore (i.e. assume defined)
    #[serde(default)]
    pub symbols: Vec<String>,

    /// Paths to symbol dump file that contains dynamic symbols accessible by the module.
    ///
    /// Supports both `objdump -T` (including `llvm-objdump`) and `llvm-nm --dynamic`
    #[serde(default)]
    pub symbol_files: Vec<PathBuf>,

    /// Regex patterns to disallow in the output of `objdump -d`
    #[serde(default)]
    pub disallowed_instructions: Vec<cu::Regex>,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}

impl Resolve for CheckConfig {
    fn resolve(&mut self, project: &ProjectTargetEnv, _: Option<&ToolchainEnv>) -> cu::Result<()> {
        for p in &mut self.symbol_files {
            *p = project.root.join(&*p);
        }
        Ok(())
    }
}

impl Validate for CheckConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)
    }
}

impl ExtendProfile for CheckConfig {
    fn extend_profile(&mut self, other: &Self) {
        self.symbols.extend(other.symbols.iter().cloned());
        self.symbol_files.extend(other.symbol_files.iter().cloned());
        self.disallowed_instructions
            .extend(other.disallowed_instructions.iter().cloned());
    }
}
