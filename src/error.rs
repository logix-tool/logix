use std::fmt;

use logix_type::error::ParseError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Failed to locate the home directory")]
    LocateHome,
    #[error("{0:?}")]
    ParseError(#[from] ParseError),
    #[error("Failed to walk path: {0}")]
    WalkDir(walkdir::Error),
    #[error("An unexpected error occured when stripping a prefix of a path previously joined with said prefix")]
    StripPrefixFailed,
    #[error("Failed to read file for diff: {0}")]
    ReadForDiff(std::io::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{self}")
    }
}
