use std::path::Path;

use cu::pre::*;

mod ftp_client;
mod min_config;

#[derive(clap::Parser, AsRef)]
struct Cli {
    #[clap(subcommand)]
    command: FtpCommand,

    #[clap(flatten)]
    #[as_ref]
    common: cu::cli::Flags,
}

impl Cli {
    pub fn preprocess(&mut self) {
        self.common.merge(self.command.as_ref());
    }
}

#[derive(clap::Parser)]
enum FtpCommand {
    /// Upload the built artifact
    Upload {
        #[clap(flatten)]
        common: CommonOptions,
    },
    /// Download crash reports and logs
    Download {
        /// Also delete the logs on the remote after downloading them
        #[clap(short = 'd', long)]
        delete: bool,

        #[clap(flatten)]
        common: CommonOptions,
    },
}

impl AsRef<cu::cli::Flags> for FtpCommand {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            FtpCommand::Upload { common, .. } => common.as_ref(),
            FtpCommand::Download { common, .. } => common.as_ref(),
        }
    }
}

#[derive(clap::Parser, AsRef)]
struct CommonOptions {
    /// Specify the location of Megaton.toml
    ///
    /// Unlike the main cli, currently we only search Megaton.toml in current dir.
    /// This will change when integrating the ftp CLI into the main CLI
    #[clap(short = 'c', long, default_value = "Megaton.toml")]
    config: String,

    /// The profile for selecting the upload artifact
    #[clap(short = 'p', long, default_value = "none")]
    profile: String,

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

#[cu::cli(preprocess = Cli::preprocess)]
async fn main(args: Cli) -> cu::Result<()> {
    let (common, keep, is_download) = match args.command {
        FtpCommand::Upload { common } => (common, false, false),
        FtpCommand::Download { common, delete } => (common, !delete, true),
    };
    // find megaton.toml
    let config = cu::fs::read_string(&common.config)?;
    let config = toml::parse::<min_config::Config>(&config)?;
    let module = config.module;

    let profile = common.profile;

    // input/output:
    // <config.target>/megaton/<profile>/<name>/[crash_reports|logs|main.npdm|<name>.nso]
    let target_dir = module.target.as_deref().unwrap_or("target");
    let base_dir = cu::path!(&(Path::new(target_dir)) / "megaton" / profile / module.name);
    let server = cu::check!(get_server(common.server), "failed to get server address")?;

    if is_download {
        cu::fs::make_dir(&base_dir)?;
        let mut ftp = ftp_client::FtpClient::connect(&server).await?;
        let dir = cu::path!(&base_dir / "crash_reports");
        ftp.download("/atmosphere/crash_reports", dir.as_utf8()?, keep)
            .await?;
        let dir = cu::path!(&base_dir / "megaton_logs");
        let log_server_path = format!("/megaton/{}/logs", module.name);
        ftp.download(&log_server_path, dir.as_utf8()?, keep).await?;
        ftp.quit().await;
    } else {
        if !base_dir.exists() {
            cu::bail!(
                "the target directory does not exist; please ensure the '{profile}' profile is built"
            );
        }
        let npdm_path = cu::path!(&base_dir / "main.npdm");
        let npdm = cu::check!(
            cu::fs::read(npdm_path),
            "failed to read main.npdm, please ensure it is built"
        )?;
        let nso_name = format!("{}.nso", module.name);
        let nso_path = cu::path!(&base_dir / nso_name);
        let nso = cu::check!(
            cu::fs::read(nso_path),
            "failed to read {nso_name}, please ensure it is built"
        )?;
        let nso_server_name = module.nso_name.unwrap_or("subsdk9".to_string());
        let title_id = format!("{:016X}", module.title_id);
        let server_path = format!("/atmosphere/contents/{title_id}/exefs");

        let mut ftp = ftp_client::FtpClient::connect(&server).await?;
        ftp.upload(
            &server_path,
            vec![(&nso_server_name, nso), ("main.npdm", npdm)],
        )
        .await?;
        ftp.quit().await;
    }

    Ok(())
}

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
