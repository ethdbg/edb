

use failure::{Fail};

#[derive(Debug, Fail)]
pub enum EDBError {
    #[fail(display = "Unsupported file type: {}", _0)]
    FileExtensionParse(String)
}
