use jiff::{Span, ToSpan};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    cache::Cache,
    env::Env,
    error::Error,
    helpers::cargo::{CargoState, CrateSpec},
    managed_package::PackageVersion,
};

pub struct SystemState {
    cargo: CargoState,
    cache: Cache,
}

impl SystemState {
    pub fn init(env: &Env) -> Result<Self, Error> {
        Ok(Self {
            cargo: CargoState::init()?,
            cache: Cache::new(env.user_cache().join("logix")?),
        })
    }

    pub fn refresh_state(&mut self) -> Result<(), Error> {
        let Self { cargo, cache: _ } = self;
        cargo.refresh_state()?;
        Ok(())
    }

    pub fn cargo_package_version(&self, spec: &CrateSpec) -> Result<PackageVersion, Error> {
        Ok(self.cargo.package_version(spec))
    }

    pub fn cargo_latest_package_version(&self, spec: &CrateSpec) -> Result<PackageVersion, Error> {
        self.cached(
            &format!("cargo/{}.latest", spec.cache_name()),
            1.hour(),
            || self.cargo.latest_package_version(spec, self.cache()),
        )
    }

    pub fn cargo_install_package(&mut self, spec: &CrateSpec) -> Result<PackageVersion, Error> {
        self.cargo.install_package(spec)
    }

    pub fn cached<T: Serialize + DeserializeOwned>(
        &self,
        key: &str,
        ttl: Span,
        f: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.cache.get_or_insert(key, ttl, f)
    }

    pub fn cache(&self) -> &Cache {
        &self.cache
    }
}
