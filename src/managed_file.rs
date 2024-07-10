use logix_type::types::FullPath;

use crate::{based_path::BasedPath, error::Error};

pub enum CmpRes {
    UpToDate,
    Missing,
    LocalAdded,
    LogixAdded,
    Modified,
}

impl CmpRes {
    fn diff_files(a: Vec<u8>, b: Vec<u8>) -> CmpRes {
        if a == b {
            Self::UpToDate
        } else {
            Self::Modified
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LocalFile {
    pub local: BasedPath,
    pub logix: BasedPath,
}

impl LocalFile {
    fn compare_files(&self) -> Result<CmpRes, Error> {
        let Self { local, logix } = self;
        if local.exists() {
            if logix.exists() {
                let a = std::fs::read(local).map_err(Error::ReadForDiff)?;
                let b = std::fs::read(logix).map_err(Error::ReadForDiff)?;
                Ok(CmpRes::diff_files(a, b))
            } else {
                Ok(CmpRes::LocalAdded)
            }
        } else if logix.exists() {
            Ok(CmpRes::LogixAdded)
        } else {
            Ok(CmpRes::Missing)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VirtualFile {
    local: FullPath,
    content: String,
}

impl VirtualFile {
    fn compare_files(&self) -> Result<CmpRes, Error> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum ManagedFile {
    Local(LocalFile),
    Virtual(VirtualFile),
    Recommend(LocalFile),
}

impl ManagedFile {
    pub fn compare_files(&self) -> Result<CmpRes, Error> {
        match self {
            Self::Local(file) => file.compare_files(),
            Self::Virtual(file) => file.compare_files(),
            Self::Recommend(file) => file.compare_files(),
        }
    }
}
