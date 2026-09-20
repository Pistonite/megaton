
use cu::pre::*;

use megaton_config::{AsProfileFlag, BASE_PROFILE};

/// Subcommands for the `ftp` command
#[derive(clap::Parser)]
pub enum SubCmd {
    /// Upload the configured files [default: the built artifacts]
    Upload {
        #[clap(flatten)]
        common: CommonOpts,
    },
    /// Download the configured files [default: crash reports and logs]
    Download {
        /// Also delete the logs on the remote after downloading them
        #[clap(short = 'd', long)]
        delete: bool,

        #[clap(flatten)]
        common: CommonOpts,
    },
}

impl AsProfileFlag for SubCmd {
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>> {
        match self {
            SubCmd::Upload { common } => common.as_profile_mut(),
            SubCmd::Download { common, .. } => common.as_profile_mut()
        }
    }
}

#[derive(clap::Parser, AsRef)]
pub struct CommonOpts {
    /// The profile for selecting what to upload or download
    #[clap(short = 'p', long)]
    profile: Option<String>,

    /// The FTP server location, in the form of <hostname>[:<port>].
    ///
    /// If port is omitted, it defaults to 5000, which is the default for ftpd.
    /// You can also use the MEGATON_NS_SERVER environment variable to set this
    #[clap(short = 's', long)]
    server: Option<String>,

    #[clap(flatten)]
    #[as_ref]
    common: cu::cli::Flags,
}

impl AsProfileFlag for CommonOpts {
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>> {
        Some(&mut self.profile)
    }
}
