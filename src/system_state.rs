use std::cell::RefCell;

use jiff::{Span, ToSpan};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    cache::Cache, env::Env, error::Error, helpers::cargo::CargoState,
    managed_package::PackageVersion,
};

pub struct SystemState {
    cargo: CargoState,
    cache: RefCell<Cache>,
}

impl SystemState {
    pub fn init(env: &Env) -> Result<Self, Error> {
        Ok(Self {
            cargo: CargoState::init()?,
            cache: RefCell::new(Cache::new(env.user_cache().join("logix")?)),
        })
    }

    pub fn refresh_state(&mut self) -> Result<(), Error> {
        let Self { cargo, cache: _ } = self;
        cargo.refresh_state()?;
        Ok(())
    }

    pub fn cargo_package_version(&self, name: &str) -> Result<PackageVersion, Error> {
        Ok(self.cargo.package_version(name))
    }

    pub fn cargo_latest_package_version(&self, name: &str) -> Result<PackageVersion, Error> {
        self.cached(&format!("cargo/crates.io/{name}.latest"), 1.hour(), || {
            self.cargo.latest_package_version(name)
        })
    }

    pub fn cached<T: Serialize + DeserializeOwned>(
        &self,
        key: &str,
        ttl: Span,
        f: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.cache.borrow_mut().get_or_insert(key, ttl, f)
    }
}
