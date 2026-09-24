use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use cu::pre::*;

use megaton_config::cmd::AsProfileFlag;
use megaton_config::config_file::{Config, ConfigLoadOpts, FtpConfig};

use crate::client::FtpClient;

/// The `ftp` command
#[derive(clap::Parser, AsRef)]
pub struct Cmd {
    #[clap(subcommand)]
    command: SubCmd,
}

impl Cmd {
    pub async fn run(self, dir: Option<&str>) -> cu::Result<()> {
        match self.command {
            SubCmd::Upload { common } => {
                let config = common.load_config(dir)?;
                let ftp_config = config.ftp_config()?;
                let profile = &config.project.profile;
                let server = get_server(common.server)?;
                cu::check!(run_upload(&server, profile, &ftp_config).await, "upload failed")?;
            }
            SubCmd::Download { delete, common } => {
                let config = common.load_config(dir)?;
                let ftp_config = config.ftp_config()?;
                let profile = &config.project.profile;
                let server = get_server(common.server)?;
                cu::check!(run_upload(&server, profile, &ftp_config).await, "upload failed")?;
            }
        }
        Ok(())
    }
}

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
impl CommonOpts {
    
    fn load_config(&self, dir: Option<&str>) -> cu::Result<Config> {
        let opts = ConfigLoadOpts {
            resolve: true,
            validate: true,
            cli_profile: self.profile.as_deref(),
            toolchain: None, // ftp tool does not need the toolchain to be resolved
        };
        Config::new_from_project_dir(dir, opts)
    }
}

impl AsProfileFlag for CommonOpts {
    fn as_profile_mut(&mut self) -> Option<&mut Option<String>> {
        Some(&mut self.profile)
    }
}

async fn run_upload(
    server: &str, profile: &str, config: &FtpConfig
) -> cu::Result<()> {
    const BIG_FILE: usize = 0x4000000; // 64MB
    let mut upload_path_maps = BTreeMap::<String, BTreeMap<String, Vec<u8>>>::new();
    let mut count = 0;
    for task in config.resolved_upload_tasks() {
        let local_path = &task.local_path;
        if !local_path.exists() {
            cu::bail!(
                "the file to upload does not exist; please ensure the '{profile}' profile is built: '{}'",
                local_path.display()
            );
        }
        let content = cu::fs::read(local_path)?;
        let server_path = Path::new(&task.server_path);
        let name = cu::check!(server_path.file_name_str(), "invalid server file path; must be the target file and not directory: '{}'", server_path.display())?;
        let directory = cu::check!(server_path.parent(), "invalid server file path; must be the target file and not directory: '{}'", server_path.display())?;
        let directory = directory.as_utf8()?;

        if content.len() >= BIG_FILE {
            let mut ftp = FtpClient::connect(&server).await?;
            ftp.upload(directory, vec![(name.to_string(), content)]).await?;
            ftp.quit().await;
            count += 1;
            continue;
        }

        {
            use std::collections::btree_map::Entry;
            match upload_path_maps.entry(directory.to_string()) {
                Entry::Vacant(e) => {
                    e.insert(std::iter::once((name.to_string(), content)).collect());
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().insert(name.to_string(), content);
                }
            }
        }
    }

    let mut ftp = FtpClient::connect(&server).await?;
    for (dir, files) in upload_path_maps {
        let files = files.into_iter().collect::<Vec<_>>();
        count += files.len();
        ftp.upload(&dir, files).await?;
    }
    cu::info!("{count} files uploaded successfully");
    ftp.quit().await;
    Ok(())
}

async fn run_download(
    server: &str, config: &FtpConfig, keep: bool
) -> cu::Result<()> {
    cu::fs::make_dir(&base_dir)?;
    let mut ftp = FtpClient::connect(&server).await?;
    for task in config.resolved_download_tasks() {
        let local_path = task.local_path;
    }
    let dir = cu::path!(&base_dir / "crash_reports");
    ftp.download("/atmosphere/crash_reports", dir.as_utf8()?, keep)
    .await?;
    let dir = cu::path!(&base_dir / "megaton_logs");
    let log_server_path = format!("/megaton/{}/logs", module.name);
    ftp.download(&log_server_path, dir.as_utf8()?, keep).await?;
    ftp.quit().await;
    todo!()
}

#[cu::context("failed to get server address")]
fn get_server(server: Option<String>) -> cu::Result<String> {
    let mut server = server.unwrap_or_default();
    if server.is_empty() {
        server = cu::env_var("MEGATON_NS_SERVER")?;
        if server.is_empty() {
            cu::bail!(
                "please set the server (hostname[:port]) using the --server flag or MEGATON_NS_SERVER environment variable"
            );
        }
    }
    // parse the optional port if any
    let (hostname, port) = match server.rfind(':') {
        None => (server.as_str(), None),
        Some(i) => {
            let port = &server[i + 1..];
            if port.chars().any(|c| !c.is_numeric()) {
                (server.as_str(), None)
            } else {
                (&server[..i], Some(port))
            }
        }
    };
    let port = port.unwrap_or("5000");
    Ok(format!("{hostname}:{port}"))
}
