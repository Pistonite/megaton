use buildcommon::prelude::*;

use buildcommon::env::Env;

use crate::cli::TopLevelOptions;
use crate::error::Error;

pub fn run(top: &TopLevelOptions) -> Result<(), Error> {
    match Env::check(top.home.as_deref()) {
        Err(e) => {
            errorln!("Failed", "Error occured during environment check");

            Err(e.change_context(Error::CheckEnv))
        }
        Ok(None) => {
            errorln!("Failed", "Environment check");
            hintln!("Consider", "Fix the issues above and try again");

            Err(report!(Error::CheckEnv))
        }
        Ok(Some(env)) => {
            infoln!("Success", "Environment check OK");
            env.save();

            Ok(())
        }
    }
}
