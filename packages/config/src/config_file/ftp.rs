use cu::pre::*;

use crate::config_file::{self, CaptureUnused, DefaultToken, ExtendProfile, Validate, ValidateCtx};

/// `[ftp]` config section
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FtpConfig {
    /// Upload tasks
    uploads: Option<Vec<FtpUploadTask>>,
    /// Download tasks
    downloads: Option<Vec<FtpDownloadTask>>,

    #[serde(flatten, default)]
    unused: CaptureUnused,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FtpUploadTask {
    #[serde(rename = "<default>")]
    Default,
    Npdm,
    Subsdk,
    #[serde(untagged)]
    Custom(FtpTask)
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
    #[serde(untagged)]
    Custom(FtpTask)
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
    local: String,
    /// Path on the server
    server: String,
}



#[cfg(test)]
mod tests {
    use super::*;

    use crate::config_file::DEFAULT_TOKEN;

    #[test]
    fn default_token_tasks() -> cu::Result<()> {
        // the `rename` on the `Default` variants must stay in sync with the token
        let token = format!("\"{DEFAULT_TOKEN}\"");
        assert_eq!(json::parse::<FtpUploadTask>(&token)?, FtpUploadTask::Default);
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
        assert_eq!(json::parse::<FtpUploadTask>("\"npdm\"")?, FtpUploadTask::Npdm);
        assert_eq!(
            json::parse::<FtpUploadTask>("\"subsdk\"")?,
            FtpUploadTask::Subsdk
        );
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
                local: "target/foo.nso".to_string(),
                server: "/atmosphere/foo.nso".to_string(),
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
                local: "crash".to_string(),
                server: "/atmosphere/crash_reports".to_string(),
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
                    FtpUploadTask::Subsdk,
                    FtpUploadTask::Custom(FtpTask {
                        local: "a".to_string(),
                        server: "b".to_string(),
                    })
                ]),
                downloads: Some(vec![
                    FtpDownloadTask::Default,
                    FtpDownloadTask::CrashReports
                ]),
                unused: Default::default()
            }
        );
        assert_eq!(json::parse::<FtpConfig>("{}")?, FtpConfig::default());
        Ok(())
    }
}
