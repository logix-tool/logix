use std::{fmt, path::PathBuf, sync::Arc};

use logix_type::{
    error::{ParseError, PathError},
    types::FullPath,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Failed to determine the {0} directory")]
    LocateDir(&'static str),

    #[error("Invalid {0} directory: {1}")]
    InvalidDir(&'static str, PathError),

    #[error("{0:?}")]
    ParseError(#[from] ParseError),

    #[error("Failed to walk path: {0}")]
    WalkDir(walkdir::Error),

    #[error("An unexpected error occured when stripping a prefix of a path previously joined with said prefix")]
    StripPrefixFailed,

    #[error("Failed to read file for diff: {0}")]
    ReadForDiff(std::io::Error),

    #[error("Failed to join local directory: {0}")]
    JoinLocal(PathError),

    #[error("Failed to join logix directory: {0}")]
    JoinLogix(PathError),

    #[error("Failed to join path: {0}")]
    JoinPath(PathError),

    #[error("The path {1:?} does not contain the base {0:?}")]
    PathNotBasedOn(Arc<FullPath>, PathBuf),

    #[error("Failed to extract file name from path {0:?}")]
    GetFileName(PathBuf),

    #[error("File name is not valid utf-8 for the path {0:?}")]
    FileNameToStr(PathBuf),

    #[error("File name is not a valid dotfile, it must start with `.` for the path {0:?}")]
    FileNameNotDotfile(PathBuf),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{self}")
    }
}
