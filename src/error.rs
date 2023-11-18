use std::fmt;

use logix_type::error::ParseError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Failed to locate the home directory")]
    LocateHome,
    #[error("{0:?}")]
    ParseError(#[from] ParseError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{self}")
    }
}
