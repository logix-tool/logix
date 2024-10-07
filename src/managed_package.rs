use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    config::Package, error::Error, github::GitHubRepo, helpers::cargo::CrateSpec,
    system_state::SystemState,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PackageVersion {
    /// The package does not exist
    None,
    Commit {
        id: String,
        date: jiff::Timestamp,
    },
    Semver(semver::Version),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackageStatus {
    /// The version installed locally
    pub installed_version: PackageVersion,
    /// The version of the local sources
    pub downloaded_version: PackageVersion,
    /// The version of the latest files on the internet
    pub latest_version: PackageVersion,
}

impl PackageStatus {
    pub fn need_update(&self) -> bool {
        if self.latest_version != self.installed_version {
            match self.latest_version {
                PackageVersion::None => false,
                PackageVersion::Commit { .. } | PackageVersion::Semver(_) => true,
            }
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedPackage<'a> {
    name: Arc<str>,
    package: &'a Package,
}

impl<'a> ManagedPackage<'a> {
    pub fn new(name: &Arc<str>, package: &'a Package) -> Self {
        Self {
            name: name.clone(),
            package,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn calculate_status(&self, state: &SystemState) -> Result<PackageStatus, Error> {
        match self.package {
            Package::RustCrate {
                crate_name,
                source,
                config_dir: _,
                environment: _,
            } => {
                let crate_spec =
                    CrateSpec::new(crate_name.as_deref().unwrap_or(&self.name), source.as_ref());

                Ok(PackageStatus {
                    installed_version: state.cargo_package_version(&crate_spec)?,
                    downloaded_version: PackageVersion::None,
                    latest_version: state.cargo_latest_package_version(&crate_spec)?,
                })
            }
            Package::Custom {
                source,
                local_dir: _,  // TODO: Use this for downloaded_version
                config_dir: _, // TODO: Use this for downloaded_version
            } => Ok(PackageStatus {
                installed_version: PackageVersion::None,
                downloaded_version: PackageVersion::None,
                latest_version: match source {
                    crate::config::Source::GitHub { owner, repo } => {
                        let gh = GitHubRepo::new(owner, repo, state.cache());
                        let info = gh.get_info()?;
                        let branch = gh.get_branch_info(&info.default_branch)?;
                        PackageVersion::Commit {
                            id: branch.commit.sha,
                            date: branch.commit.commit.author.date,
                        }
                    }
                },
            }),
        }
    }

    pub fn install_update(&self, state: &mut SystemState) -> Result<PackageVersion, Error> {
        match self.package {
            Package::RustCrate {
                crate_name,
                source,
                config_dir: _,
                environment: _,
            } => {
                let crate_spec =
                    CrateSpec::new(crate_name.as_deref().unwrap_or(&self.name), source.as_ref());
                state.cargo_install_package(&crate_spec)
            }
            Package::Custom { .. } => todo!(),
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self.package, Package::Custom { .. })
    }
}
