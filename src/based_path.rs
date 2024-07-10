use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use logix_type::types::FullPath;

use crate::error::Error;

/// Represents a relative path with a base, so it can be both absolute and relative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasedPath {
    base: Arc<FullPath>,
    path: Option<FullPath>,
}

impl BasedPath {
    pub fn new(base: FullPath) -> Self {
        Self {
            base: Arc::new(base),
            path: None,
        }
    }

    /// Rebase the path, making [self.as_full_path] the new base
    pub fn rebased(self) -> BasedPath {
        Self {
            base: self.path.map(Arc::new).unwrap_or(self.base),
            path: None,
        }
    }

    /// Return the base path
    pub fn as_base_path(&self) -> &FullPath {
        &self.base
    }

    /// Return the entire path as [Path]
    pub fn as_path(&self) -> &Path {
        self.path.as_deref().unwrap_or(&self.base)
    }

    /// Return the entire path as [FullPath]
    pub fn as_full_path(&self) -> &FullPath {
        self.path.as_ref().unwrap_or(&self.base)
    }

    /// Return the relative part of this path
    pub fn rel_path(&self) -> &Path {
        self.path
            .as_ref()
            .map(|p| p.strip_prefix(self.base.as_ref()).unwrap())
            .unwrap_or(Path::new(""))
    }

    /// Join the current path with the specified relative path, if the path isn't
    /// relative, this method will fail
    pub fn join(&self, rel_path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            base: self.base.clone(),
            path: self
                .as_full_path()
                .join(rel_path)
                .map(Some)
                .map_err(Error::JoinPath)?,
        })
    }

    /// Set the internal path to the exact path specified, if it doesn't start with
    /// the base path as a prefix, this method will fail
    pub fn with_path_buf(&self, path: PathBuf) -> Result<Self, Error> {
        if path.strip_prefix(self.base.as_path()).is_ok() {
            Ok(Self {
                base: self.base.clone(),
                path: Some(FullPath::try_from(path).unwrap()),
            })
        } else {
            Err(Error::PathNotBasedOn(self.base.clone(), path))
        }
    }
}

impl std::ops::Deref for BasedPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl AsRef<Path> for BasedPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<OsStr> for BasedPath {
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl From<BasedPath> for PathBuf {
    fn from(value: BasedPath) -> Self {
        value
            .path
            .unwrap_or_else(|| value.base.as_ref().clone())
            .into()
    }
}
