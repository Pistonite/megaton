pub mod compdb;
pub mod env;
pub mod flags;
pub mod print;
pub mod source;
pub mod system;

mod unused;
pub use unused::Unused;

pub mod prelude {
    pub use crate::system;
    pub use crate::system::{PathExt, ResultIn, ResultInExt};
    pub use crate::{args, error_context, errorln, hintln, infoln, verboseln};
    pub use error_stack::{bail, report, Report, Result, ResultExt};
}
