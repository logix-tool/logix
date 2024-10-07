use std::{
    cell::RefCell,
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
};

use jiff::{Span, Timestamp, ToSpan, Unit};
use logix_vfs::{LogixVfs, RelFs};
use serde::{de::DeserializeOwned, Serialize};

use crate::{based_path::BasedPath, error::Error};

struct Inner {
    fs: RelFs,
    base: BasedPath,
    cache: RefCell<HashMap<String, Vec<u8>>>,
}

#[derive(Clone)]
pub struct Cache {
    inner: Rc<Inner>,
}

impl Cache {
    pub fn new(dir: BasedPath) -> Self {
        Self {
            inner: Rc::new(Inner {
                fs: RelFs::new(&dir),
                base: dir,
                cache: Default::default(),
            }),
        }
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, logix_vfs::Error> {
        let mut ret = Vec::new();
        self.inner
            .fs
            .open_file(path)?
            .read_to_end(&mut ret)
            .map_err(|e| logix_vfs::Error::from_io(path.to_path_buf(), e))?;
        Ok(ret)
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), logix_vfs::Error> {
        let path = self
            .inner
            .base
            .join(self.inner.fs.canonicalize_path(path)?)
            .unwrap();
        // TODO: Add this functionality to RelFs itself?
        std::fs::create_dir_all(&path).map_err(|e| logix_vfs::Error::from_io(path.to_path_buf(), e))
    }

    fn write_file(&self, path: &Path, data: &[u8]) -> Result<(), logix_vfs::Error> {
        let path = self
            .inner
            .base
            .join(self.inner.fs.canonicalize_path(path)?)
            .unwrap();
        // TODO: Add this functionality to RelFs itself?
        std::fs::write(&path, data).map_err(|e| logix_vfs::Error::from_io(path.to_path_buf(), e))
    }

    fn modified_time(&self, path: &Path) -> Result<Timestamp, logix_vfs::Error> {
        let path = self
            .inner
            .base
            .join(self.inner.fs.canonicalize_path(path)?)
            .unwrap();
        // TODO: Add this functionality to RelFs itself?
        let ts = std::fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .map_err(|e| logix_vfs::Error::from_io(path.to_path_buf(), e))?;
        Ok(ts.try_into().unwrap())
    }

    pub fn get_or_insert<T: Serialize + DeserializeOwned>(
        &self,
        key: &str,
        ttl: Span,
        f: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        if let Some(ret) = self.inner.cache.borrow().get(key) {
            log::debug!("Found {key:?} in memory");
            // NOTE: This shouldn't ever fail as we just serialized it within the same process
            return Ok(serde_json::from_slice(ret).unwrap());
        }

        let path = PathBuf::from(format!("{key}.json"));

        log::trace!("Looking for cached entry for {key:?} at {path:?}");

        if let Ok(modified) = self.modified_time(&path) {
            let ttl = (ttl.total(Unit::Minute).unwrap() as i64).minutes();
            let expires_at = modified + ttl;

            log::trace!("Cached entry for {key:?} at {path:?} is last modified at {modified} and expires at {expires_at}");

            if Timestamp::now() < expires_at {
                log::debug!("Using cached entry for {key:?} at {path:?} last modified at {modified} and expires at {expires_at}");

                match self.read_file(&path) {
                    Ok(data) => match serde_json::from_slice(&data) {
                        Ok(ret) => {
                            self.inner.cache.borrow_mut().insert(key.into(), data);
                            return Ok(ret);
                        }
                        Err(e) => {
                            log::warn!("Failed to load cached item: {e}");
                        }
                    },
                    Err(e) => {
                        log::warn!("Failed to read cached item: {e}");
                    }
                }
            }
        }

        log::debug!("Fetching new entry for {key:?} at {path:?}");

        let ret = f()?;

        match self.create_dir_all(path.parent().unwrap()) {
            Ok(_) => match serde_json::to_vec_pretty(&ret) {
                Ok(data) => match self.write_file(&path, &data) {
                    Ok(_) => {
                        self.inner.cache.borrow_mut().insert(key.into(), data);
                    }
                    Err(e) => {
                        log::warn!("Failed to store cached item to {path:?}: {e}");
                    }
                },
                Err(e) => {
                    log::warn!("Failed to serialize cached item {path:?}: {e}");
                }
            },
            Err(e) => {
                log::warn!("Failed to create directory for {path:?}: {e}");
            }
        }

        Ok(ret)
    }
}
