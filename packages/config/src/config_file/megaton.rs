use cu::pre::*;
use semver::{Version, VersionReq};

use crate::config_file::{Validate, ValidateCtx, CaptureUnused};

/// The `[megaton]` section
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MegatonConfig {
    /// If specified, megaton will run version check and error if the current
    /// version doesn't match the given requirement.
    pub version: Option<VersionReq>,

    #[serde(flatten, default, skip_serializing)]
    unused: CaptureUnused,
}

impl Validate for MegatonConfig {
    fn validate(&self, ctx: &mut ValidateCtx) -> cu::Result<()> {
        if let Some(v) = &self.version {
            check_megaton_version_requirement(v)?
        }
        self.unused.validate(ctx)
    }
}

/// Check the current buildtool/library version matches the requirement
pub fn check_megaton_version_requirement(version_req: &VersionReq) -> cu::Result<()> {
    let curr_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    if !version_req.matches(&curr_version) {
        cu::bail!(
            "the project requires megaton {version_req}, but current version is {curr_version}"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {

use super::*;

    #[test]
    fn cargo_pkg_version_is_valid_semver() -> cu::Result<()> {
        Version::parse(env!("CARGO_PKG_VERSION"))?;
        VersionReq::parse(concat!("^", env!("CARGO_PKG_VERSION")))?;
        Ok(())
    }

    #[test]
    fn version_req_wildcard_matches() -> cu::Result<()> {
        let req = VersionReq::parse("*")?;
        assert!(check_megaton_version_requirement(&req).is_ok());
        Ok(())
    }

    #[test]
    fn version_req_impossible_fails() -> cu::Result<()> {
        let curr = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let req = VersionReq::parse(">=99999.0.0")?;
        let result = check_megaton_version_requirement(&req);
        assert!(result.is_err());
        let error_message = format!("{:?}", result.unwrap_err());
        assert_eq!(
            error_message,
            format!("the project requires megaton version >=99999.0.0, current version is {curr}")
        );
        Ok(())
    }
}
