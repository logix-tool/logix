use crate::based_path::BasedPath;

/// Represents the status of a given file
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileStatus {
    UpToDate,
    MissingFromBoth,
    LocalAdded,
    LogixAdded,
    Modified,
    ErrorReadingLocal(std::io::ErrorKind),
    ErrorReadingLogix(std::io::ErrorKind),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct LocalFile {
    pub local: BasedPath,
    pub logix: BasedPath,
}

impl LocalFile {
    fn calculate_status(&self) -> FileStatus {
        let Self { local, logix } = self;
        if local.exists() {
            if logix.exists() {
                let a = match std::fs::read(local) {
                    Ok(a) => a,
                    Err(e) => return FileStatus::ErrorReadingLocal(e.kind()),
                };
                let b = match std::fs::read(logix) {
                    Ok(b) => b,
                    Err(e) => return FileStatus::ErrorReadingLogix(e.kind()),
                };
                if a == b {
                    FileStatus::UpToDate
                } else {
                    FileStatus::Modified
                }
            } else {
                FileStatus::LocalAdded
            }
        } else if logix.exists() {
            FileStatus::LogixAdded
        } else {
            FileStatus::MissingFromBoth
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Clone)]
pub struct VirtualFile {
    local: BasedPath,
    content: String,
}

impl VirtualFile {
    fn calculate_status(&self) -> FileStatus {
        let Self { local, content } = self;
        if local.exists() {
            let a = match std::fs::read(local) {
                Ok(a) => a,
                Err(e) => return FileStatus::ErrorReadingLocal(e.kind()),
            };
            if a == content.as_bytes() {
                FileStatus::UpToDate
            } else {
                FileStatus::Modified
            }
        } else {
            FileStatus::LogixAdded
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Clone)]
pub enum ManagedFile {
    Local(LocalFile),
    Virtual(VirtualFile),
}

impl ManagedFile {
    pub fn calculate_status(&self) -> FileStatus {
        match self {
            Self::Local(file) => file.calculate_status(),
            Self::Virtual(file) => file.calculate_status(),
        }
    }

    pub fn local_path(&self) -> Option<&BasedPath> {
        match self {
            Self::Local(file) => Some(&file.local),
            Self::Virtual(file) => Some(&file.local),
        }
    }

    pub fn logix_path(&self) -> Option<&BasedPath> {
        match self {
            Self::Local(file) => Some(&file.logix),
            Self::Virtual(_) => None,
        }
    }
}
