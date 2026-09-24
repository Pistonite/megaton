use std::path::PathBuf;

use cu::pre::*;

use crate::config_file::{
    self, CaptureUnused, DefaultToken, ExtendProfile, ProjectTargetEnv, Resolve, Validate,
    ValidateCtx,
};
use crate::toolchain::ToolchainEnv;

/// `[ftp]` config section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FtpConfig {
    /// Upload tasks
    uploads: Option<Vec<FtpUploadTask>>,
    /// Download tasks
    downloads: Option<Vec<FtpDownloadTask>>,

    #[serde(skip_deserializing)]
    resolved: ResolvedFtpTasks,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}
impl FtpConfig {
    #[inline(always)]
    pub fn resolved_upload_tasks(&self) -> &[FtpTask] {
        &self.resolved.uploads
    }
    #[inline(always)]
    pub fn resolved_download_tasks(&self) -> &[FtpTask] {
        &self.resolved.downloads
    }
}
impl Validate for FtpConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        self.unused.validate(ctx)?;
        Ok(())
    }
}
impl ExtendProfile for FtpConfig {
    fn extend_profile(&mut self, other: &Self) {
        config_file::extend_by_default_token(&mut self.uploads, other.uploads.as_ref());
        config_file::extend_by_default_token(&mut self.downloads, other.downloads.as_ref());
    }
}
impl Resolve for FtpConfig {
    fn resolve(
        &mut self,
        project: &ProjectTargetEnv,
        _: Option<&ToolchainEnv>,
    ) -> cu::Result<()> {
        let resolved = config_file::resolve_default_token(vec![], self.uploads.as_deref(), || {
            vec![FtpUploadTask::Npdm, FtpUploadTask::Nso]
        });
        cu::debug!("{resolved:?}");
        self.resolved.uploads = resolved
            .into_iter()
            .filter_map(|x| x.into_task(project))
            .collect();
        let resolved = config_file::resolve_default_token(vec![], self.downloads.as_deref(), || {
            vec![FtpDownloadTask::CrashReports, FtpDownloadTask::Logs]
        });
        self.resolved.downloads = resolved
            .into_iter()
            .filter_map(|x| x.into_task(project))
            .collect();

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResolvedFtpTasks {
    pub uploads: Vec<FtpTask>,
    pub downloads: Vec<FtpTask>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FtpUploadTask {
    #[serde(rename = "<default>")]
    Default,
    Npdm,
    Nso,
    #[serde(untagged)]
    Custom(FtpTask),
}
impl FtpUploadTask {
    fn into_task(self, project: &ProjectTargetEnv) -> Option<FtpTask> {
        match self {
            Self::Default => None,
            Self::Npdm => {
                let title_id = project.title_id;
                let server = format!("/atmosphere/contents/{title_id:016X}/exefs/main.npdm");
                let local = project.out_npdm.clone();
                Some(FtpTask { local_path: local, server_path: server })
            }
            Self::Nso => {
                let title_id = project.title_id;
                let nso_name = project.nso_name.as_ref()?;
                let server = format!("/atmosphere/contents/{title_id:016X}/exefs/{nso_name}");
                let local = project.out_nso.clone();
                Some(FtpTask { local_path: local, server_path: server })
            }
            Self::Custom(t) => Some(t),
        }
    }
}
impl DefaultToken for FtpUploadTask {
    fn is_default_token(&self) -> bool {
        matches!(self, Self::Default)
    }
    fn get_default_token() -> Self {
        Self::Default
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FtpDownloadTask {
    #[serde(rename = "<default>")]
    Default,
    CrashReports,
    Logs,
    #[serde(untagged)]
    Custom(FtpTask),
}
impl FtpDownloadTask {
    fn into_task(self, project: &ProjectTargetEnv) -> Option<FtpTask> {
        match self {
            Self::Default => None,
            Self::CrashReports => {
                let local = project.output_root.join("crash_reports");
                let server = "/atmosphere/crash_reports".to_string();
                Some(FtpTask { local_path: local, server_path: server })
            }
            Self::Logs => {
                let local = project.output_root.join("logs");
                let server = format!("/megaton/{}/logs", project.module_name);
                Some(FtpTask { local_path: local, server_path: server })
            }
            Self::Custom(t) => Some(t),
        }
    }
}
impl DefaultToken for FtpDownloadTask {
    fn is_default_token(&self) -> bool {
        matches!(self, Self::Default)
    }
    fn get_default_token() -> Self {
        Self::Default
    }
}
/// A custom ftp upload or download task
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FtpTask {
    /// Path on the local system
    #[serde(rename = "local")]
    pub local_path: PathBuf,
    /// Path on the server
    #[serde(rename = "server")]
    pub server_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config_file::DEFAULT_TOKEN;

    #[test]
    fn default_token_tasks() -> cu::Result<()> {
        // the `rename` on the `Default` variants must stay in sync with the token
        let token = format!("\"{DEFAULT_TOKEN}\"");
        assert_eq!(
            json::parse::<FtpUploadTask>(&token)?,
            FtpUploadTask::Default
        );
        assert_eq!(
            json::parse::<FtpDownloadTask>(&token)?,
            FtpDownloadTask::Default
        );
        // the token is the only spelling of the default task
        assert!(json::parse::<FtpUploadTask>("\"default\"").is_err());
        assert!(json::parse::<FtpDownloadTask>("\"default\"").is_err());
        Ok(())
    }

    #[test]
    fn upload_builtin_tasks() -> cu::Result<()> {
        assert_eq!(
            json::parse::<FtpUploadTask>("\"npdm\"")?,
            FtpUploadTask::Npdm
        );
        assert_eq!(json::parse::<FtpUploadTask>("\"nso\"")?, FtpUploadTask::Nso);
        Ok(())
    }

    #[test]
    fn upload_custom_task() -> cu::Result<()> {
        let task = json::parse::<FtpUploadTask>(
            r#"{ "local": "target/foo.nso", "server": "/atmosphere/foo.nso" }"#,
        )?;
        assert_eq!(
            task,
            FtpUploadTask::Custom(FtpTask {
                local_path: "target/foo.nso".to_string().into(),
                server_path: "/atmosphere/foo.nso".to_string(),
            })
        );
        Ok(())
    }

    #[test]
    fn upload_invalid_input() -> cu::Result<()> {
        // unknown built-in task
        assert!(json::parse::<FtpUploadTask>("\"crash-reports\"").is_err());
        // built-in task names are kebab-case, not the variant name
        assert!(json::parse::<FtpUploadTask>("\"Npdm\"").is_err());
        // custom task is missing `server`
        assert!(json::parse::<FtpUploadTask>(r#"{ "local": "target/foo.nso" }"#).is_err());
        Ok(())
    }

    #[test]
    fn download_builtin_tasks() -> cu::Result<()> {
        assert_eq!(
            json::parse::<FtpDownloadTask>("\"crash-reports\"")?,
            FtpDownloadTask::CrashReports
        );
        Ok(())
    }

    #[test]
    fn download_custom_task() -> cu::Result<()> {
        let task = json::parse::<FtpDownloadTask>(
            r#"{ "local": "crash", "server": "/atmosphere/crash_reports" }"#,
        )?;
        assert_eq!(
            task,
            FtpDownloadTask::Custom(FtpTask {
                local_path: "crash".to_string().into(),
                server_path: "/atmosphere/crash_reports".to_string(),
            })
        );
        Ok(())
    }

    #[test]
    fn download_invalid_input() -> cu::Result<()> {
        // upload-only built-in task
        assert!(json::parse::<FtpDownloadTask>("\"npdm\"").is_err());
        // custom task is missing `local`
        assert!(
            json::parse::<FtpDownloadTask>(r#"{ "server": "/atmosphere/crash_reports" }"#).is_err()
        );
        // extra, unknown key in a custom task is still accepted
        json::parse::<FtpDownloadTask>(r#"{ "local": "a", "server": "b", "extra": 1 }"#)?;
        Ok(())
    }

    #[test]
    fn config_with_both_task_lists() -> cu::Result<()> {
        let config = json::parse::<FtpConfig>(
            r#"{
                "uploads": ["<default>", "npdm", "subsdk", { "local": "a", "server": "b" }],
                "downloads": ["<default>", "crash-reports"]
            }"#,
        )?;
        assert_eq!(
            config,
            FtpConfig {
                uploads: Some(vec![
                    FtpUploadTask::Default,
                    FtpUploadTask::Npdm,
                    FtpUploadTask::Nso,
                    FtpUploadTask::Custom(FtpTask {
                        local_path: "a".to_string().into(),
                        server_path: "b".to_string(),
                    })
                ]),
                downloads: Some(vec![
                    FtpDownloadTask::Default,
                    FtpDownloadTask::CrashReports
                ]),
                resolved: Default::default(),
                unused: Default::default()
            }
        );
        assert_eq!(json::parse::<FtpConfig>("{}")?, FtpConfig::default());
        Ok(())
    }
}
