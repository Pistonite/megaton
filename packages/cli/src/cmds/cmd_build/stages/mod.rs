pub mod cxxbridge;

use std::path::PathBuf;
use cu::pre::*;
use crate::cmds::cmd_build::config::{Config, Flags};
use crate::cmds::cmd_build::CmdBuild;

#[derive(Debug, Default)]
pub struct StageOutputs {
    pub objects: Vec<PathBuf>,
}

impl StageOutputs {
    pub fn extend_objects<I: IntoIterator<Item = PathBuf>>(&mut self, it: I) {
        self.objects.extend(it);
    }
}

pub fn run_all(config: &Config, args: &CmdBuild, flags: &mut Flags) -> cu::Result<StageOutputs> {
    let mut out: StageOutputs = StageOutputs::default();
    let produced: Vec<PathBuf> = cxxbridge::run(config, args, flags).context("cxxbridge stage failed")?;
    out.extend_objects(produced);
    Ok(out)
}
