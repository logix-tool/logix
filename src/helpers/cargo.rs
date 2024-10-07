use std::{collections::HashMap, process::Command};

use crate::{error::Error, managed_package::PackageVersion};

#[derive(PartialEq, Eq, Hash)]
pub struct CrateName(String);

impl CrateName {
    pub fn new(name: &str) -> Self {
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

    pub fn package_version(&self, name: &str) -> PackageVersion {
        self.crates
            .get(&CrateName::new(name))
            .cloned()
            .unwrap_or(PackageVersion::None)
    }

    pub fn refresh_state(&mut self) -> Result<(), Error> {
        *self = Self::init()?;
        Ok(())
    }

    pub fn latest_package_version(&self, name: &str) -> Result<PackageVersion, Error> {
        let res = Command::new("cargo")
            .arg("search")
            .arg("--limit=1")
            .arg(name)
            .output()
            .map_err(|e| e.to_string())
            .and_then(|res| {
                if res.status.success() {
                    Ok(res.stdout)
                } else {
                    Err(format!("returned status {}", res.status))
                }
            })
            .map_err(|e| Error::ShellCommandFailed("cargo search --limit=1", e.to_string()))?;

        for line in String::from_utf8_lossy(&res).lines().take(1) {
            if let Some((got_name, remain)) = line.split_once(" = \"") {
                if let Some((version, remain)) = remain.split_once('"') {
                    if name
                        .chars()
                        .map(package_name_char)
                        .eq(got_name.chars().map(package_name_char))
                    {
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

        Ok(PackageVersion::None)
    }
}

fn parse_version(version: &str) -> PackageVersion {
    if let Ok(ver) = semver::Version::parse(version.strip_prefix('v').unwrap_or(version)) {
        PackageVersion::Semver(ver)
    } else {
        todo!("{version:?}")
    }
}

fn package_name_char(c: char) -> char {
    if c == '-' {
        '_'
    } else {
        c
    }
}

pub fn install_package(name: &str) -> Result<PackageVersion, Error> {
    Command::new("cargo")
        .arg("install")
        .arg(name)
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

    Ok(CargoState::init()?.package_version(name))
}
