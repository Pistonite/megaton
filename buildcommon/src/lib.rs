pub mod env;
pub mod flags;
pub mod print;
pub mod system;
pub mod source;

pub mod prelude {
    pub use crate::system;
    pub use crate::system::{PathExt, ResultIn, ResultInExt};
    pub use crate::{args, error_context, errorln, hintln, infoln, verboseln};
    pub use error_stack::{report, Report, Result, ResultExt};
}
