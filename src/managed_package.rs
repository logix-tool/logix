use std::sync::Arc;

use time::OffsetDateTime;

use crate::{config::Package, error::Error, github::GitHubRepo};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackageVersion {
    /// The package does not exist
    None,
    Commit {
        id: String,
        date: OffsetDateTime,
    },
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    pub fn calculate_status(&self) -> Result<PackageStatus, Error> {
        match self.package {
            Package::RustCrate {
                crate_name,
                config_dir,
            } => todo!("{crate_name:?}, {config_dir:?}"),
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
}
