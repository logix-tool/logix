use std::collections::HashMap;

use jiff::{Span, Timestamp, ToSpan, Unit};
use serde::{de::DeserializeOwned, Serialize};

use crate::{based_path::BasedPath, error::Error};

pub struct Cache {
    dir: BasedPath,
    cache: HashMap<String, Vec<u8>>,
}

impl Cache {
    pub fn new(dir: BasedPath) -> Self {
        Self {
            dir,
            cache: HashMap::new(),
        }
    }

    pub fn get_or_insert<T: Serialize + DeserializeOwned>(
        &mut self,
        key: &str,
        ttl: Span,
        f: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        if let Some(ret) = self.cache.get(key) {
            log::debug!("Found {key:?} in memory");
            // NOTE: This shouldn't ever fail as we just serialized it within the same process
            return Ok(serde_json::from_slice(ret).unwrap());
        }

        let path = self.dir.join(format!("{key}.json"))?;

        log::trace!("Looking for cached entry for {key:?} at {path:?}");

        if let Ok(modified) = path.metadata().and_then(|p| p.modified()) {
            match Timestamp::try_from(modified) {
                Ok(modified) => {
                    let ttl = (ttl.total(Unit::Minute).unwrap() as i64).minutes();
                    let expires_at = modified + ttl;

                    log::trace!("Cached entry for {key:?} at {path:?} is last modified at {modified} and expires at {expires_at}");

                    if Timestamp::now() < expires_at {
                        log::debug!("Using cached entry for {key:?} at {path:?} last modified at {modified} and expires at {expires_at}");

                        match std::fs::read(&path) {
                            Ok(data) => match serde_json::from_slice(&data) {
                                Ok(ret) => {
                                    self.cache.insert(key.into(), data);
                                    return Ok(ret);
                                }
                                Err(e) => {
                                    log::warn!("Failed to load cached item from {path:?}: {e}");
                                }
                            },
                            Err(e) => {
                                log::warn!("Failed to read cached item from {path:?}: {e}");
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to convert systemtime to timestamp: {e}");
                }
            }
        }

        log::debug!("Fetching new entry for {key:?} at {path:?}");

        let ret = f()?;

        match std::fs::create_dir_all(path.parent().unwrap()) {
            Ok(_) => match serde_json::to_vec_pretty(&ret) {
                Ok(data) => match std::fs::write(&path, &data) {
                    Ok(_) => {
                        self.cache.insert(key.into(), data);
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
