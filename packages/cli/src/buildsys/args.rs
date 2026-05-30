
use cu::pre::*;
#[derive(Debug, clap::Parser)]
pub struct BuildArgs {
    /// Select profile to build
    ///
    /// See https://megaton-new.pistonite.dev/tutorial/profiles
    #[clap(short, long, default_value = "none")]
    pub profile: String,

    /// Emit configuration files only (such as compile_commands.json),
    /// and do not actually build
    #[clap(short = 'g', long)]
    pub configure: bool,

    /// Specify the location of the config file
    #[clap(short = 'c', long, default_value = "Megaton.toml")]
    pub config: String,
}
