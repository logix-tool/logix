use std::sync::Arc;

use time::OffsetDateTime;

use crate::{
    config::Package, error::Error, github::GitHubRepo, helpers, system_state::SystemState,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackageVersion {
    /// The package does not exist
    None,
    Commit {
        id: String,
        date: OffsetDateTime,
    },
    Semver(semver::Version),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
                config_dir: _,
                environment: _,
            } => {
                let crate_name = crate_name.as_deref().unwrap_or(&self.name);

                Ok(PackageStatus {
                    installed_version: state.cargo.package_version(crate_name),
                    downloaded_version: PackageVersion::None,
                    latest_version: helpers::cargo::latest_package_version(crate_name)?,
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
                        let gh = GitHubRepo::new(owner, repo);
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

    pub fn install_update(&self) -> Result<PackageVersion, Error> {
        match self.package {
            Package::RustCrate {
                crate_name,
                config_dir: _,
                environment: _,
            } => {
                let crate_name = crate_name.as_deref().unwrap_or(&self.name);
                helpers::cargo::install_package(crate_name)
            }
            Package::Custom { .. } => todo!(),
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self.package, Package::Custom { .. })
    }
}
