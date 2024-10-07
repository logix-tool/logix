use std::{collections::HashMap, fmt, process::Command};

use crate::{
    cache::Cache, config::RustCrateSource, error::Error, github::GitHubRepo,
    managed_package::PackageVersion,
};

pub struct CrateSpec<'a> {
    name: CrateName,
    source: Option<&'a RustCrateSource>,
}

impl<'a> CrateSpec<'a> {
    pub fn new(name: &str, source: Option<&'a RustCrateSource>) -> Self {
        Self {
            name: CrateName::new(name),
            source,
        }
    }

    pub fn cache_name(&self) -> impl fmt::Display {
        match &self.source {
            Some(RustCrateSource::GithubBranch {
                owner,
                repo,
                branch,
            }) => format!(
                "github.com/{owner}/{repo}/branches/{branch}/crates/{}",
                self.name.0
            ),
            Some(RustCrateSource::GithubTag { owner, repo, tag }) => {
                format!(
                    "github.com/{owner}/{repo}/tags/{tag}/crates/{}",
                    self.name.0
                )
            }
            Some(RustCrateSource::GithubRevision {
                owner,
                repo,
                revision,
            }) => format!(
                "github.com/{owner}/{repo}/revs/{revision}/crates/{}",
                self.name.0
            ),
            None => format!("crates.io/{}", self.name.0),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct CrateName(String);

impl CrateName {
    fn new(name: &str) -> Self {
        Self(
            name.chars()
                .map(|c| if c == '_' { '-' } else { c })
                .collect(),
        )
    }
}

pub struct CargoState {
    crates: HashMap<CrateName, PackageVersion>,
}

impl CargoState {
    pub fn init() -> Result<Self, Error> {
        let mut ret = CargoState {
            crates: HashMap::new(),
        };
        let res = Command::new("cargo")
            .arg("install")
            .arg("--list")
            .output()
            .map_err(|e| e.to_string())
            .and_then(|res| {
                if res.status.success() {
                    Ok(res.stdout)
                } else {
                    Err(format!("returned status {}", res.status))
                }
            })
            .map_err(|e| Error::ShellCommandFailed("cargo install --list", e.to_string()))?;

        let re = regex::Regex::new(r"^(\S+)\s+(\S+)(\s+\([^)]+\))?:$").unwrap();

        for line in String::from_utf8_lossy(&res).lines() {
            if let Some(m) = re.captures(line) {
                if let Some(_path) = m.get(3) {
                    // TODO: Include those with a local clone?
                } else {
                    ret.crates
                        .insert(CrateName::new(&m[1]), parse_version(&m[2]));
                }
            } else if !line.starts_with(' ') {
                todo!("{line:?}")
            }
        }

        Ok(ret)
    }

    pub fn package_version(&self, spec: &CrateSpec) -> PackageVersion {
        self.crates
            .get(&spec.name)
            .cloned()
            .unwrap_or(PackageVersion::None)
    }

    pub fn refresh_state(&mut self) -> Result<(), Error> {
        *self = Self::init()?;
        Ok(())
    }

    pub fn latest_package_version(
        &self,
        spec: &CrateSpec,
        cache: &Cache,
    ) -> Result<PackageVersion, Error> {
        match spec.source {
            Some(RustCrateSource::GithubBranch {
                owner,
                repo,
                branch,
            }) => {
                let info = GitHubRepo::new(owner, repo, cache).get_branch_info(branch)?;

                return Ok(PackageVersion::Commit {
                    id: info.commit.sha,
                    date: info.commit.commit.author.date,
                });
            }
            Some(unk) => todo!("{unk:#?}"),
            None => {
                let res = Command::new("cargo")
                    .arg("search")
                    .arg("--limit=1")
                    .arg(&spec.name.0)
                    .output()
                    .map_err(|e| e.to_string())
                    .and_then(|res| {
                        if res.status.success() {
                            Ok(res.stdout)
                        } else {
                            Err(format!("returned status {}", res.status))
                        }
                    })
                    .map_err(|e| {
                        Error::ShellCommandFailed("cargo search --limit=1", e.to_string())
                    })?;

                for line in String::from_utf8_lossy(&res).lines().take(1) {
                    if let Some((got_name, remain)) = line.split_once(" = \"") {
                        if let Some((version, remain)) = remain.split_once('"') {
                            let got_name = CrateName::new(got_name);

                            if spec.name == got_name {
                                return Ok(parse_version(version));
                            } else {
                                todo!("{got_name:?}, {version:?}, {remain:?}")
                            }
                        } else {
                            todo!("{got_name:?}, {remain:?}")
                        }
                    } else {
                        todo!("{line:?}")
                    }
                }
            }
        }

        Ok(PackageVersion::None)
    }

    pub fn install_package(&mut self, spec: &CrateSpec) -> Result<PackageVersion, Error> {
        let mut cmd = Command::new("cargo");

        cmd.arg("install");

        match spec.source {
            Some(RustCrateSource::GithubBranch {
                owner,
                repo,
                branch,
            }) => {
                cmd.args(["--git", &format!("https://github.com/{owner}/{repo}.git")])
                    .args(["--branch", branch]);
            }
            Some(unk) => todo!("{unk:#?}"),
            None => {}
        }

        cmd.arg(&spec.name.0)
            .status()
            .map_err(|e| e.to_string())
            .and_then(|res| {
                if res.success() {
                    Ok(())
                } else {
                    Err(format!("returned status {res}"))
                }
            })
            .map_err(|e| Error::ShellCommandFailed("cargo install", e))?;

        self.refresh_state()?;

        Ok(self.package_version(spec))
    }
}

fn parse_version(version: &str) -> PackageVersion {
    if let Ok(ver) = semver::Version::parse(version.strip_prefix('v').unwrap_or(version)) {
        PackageVersion::Semver(ver)
    } else {
        todo!("{version:?}")
    }
}
