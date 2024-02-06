#![deny(warnings, clippy::all)]

use std::path::PathBuf;

use crate::{env::Env, error::Error, status::Status};
use env::Filter;
use logix_type::LogixLoader;
use logix_vfs::RelFs;

pub mod config;
pub mod env;
pub mod error;
pub mod status;

pub enum CmpRes {
    UpToDate,
    Missing,
    LocalAdded,
    LogixAdded,
    Modified,
}

impl CmpRes {
    fn diff_files(a: Vec<u8>, b: Vec<u8>) -> CmpRes {
        if a == b {
            Self::UpToDate
        } else {
            Self::Modified
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LocalFile {
    pub local: PathBuf,
    pub logix: PathBuf,
}

impl LocalFile {
    fn compare_files(&self) -> Result<CmpRes, Error> {
        let Self { local, logix } = self;
        if local.exists() {
            if logix.exists() {
                let a = std::fs::read(local).map_err(Error::ReadForDiff)?;
                let b = std::fs::read(logix).map_err(Error::ReadForDiff)?;
                Ok(CmpRes::diff_files(a, b))
            } else {
                Ok(CmpRes::LocalAdded)
            }
        } else if logix.exists() {
            Ok(CmpRes::LogixAdded)
        } else {
            Ok(CmpRes::Missing)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VirtualFile {
    local: PathBuf,
    content: String,
}

impl VirtualFile {
    fn compare_files(&self) -> Result<CmpRes, Error> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum ManagedFile {
    Local(LocalFile),
    Virtual(VirtualFile),
    Recommend(LocalFile),
}

impl ManagedFile {
    pub fn compare_files(&self) -> Result<CmpRes, Error> {
        match self {
            Self::Local(file) => file.compare_files(),
            Self::Virtual(file) => file.compare_files(),
            Self::Recommend(file) => file.compare_files(),
        }
    }
}

pub struct Logix {
    env: Env,
    config: config::Logix,
}

impl Logix {
    pub fn load(env: Env) -> Result<Self, Error> {
        let mut loader = LogixLoader::new(RelFs::new(env.logix_config_dir()));
        Ok(Self {
            env,
            config: loader.load_file("root.logix")?,
        })
    }

    pub fn config(&self) -> &config::Logix {
        &self.config
    }

    pub fn calculate_managed_files(&self) -> Result<Vec<ManagedFile>, Error> {
        let config::Logix { home } = &self.config;
        let mut ret = Vec::new();
        {
            let config::UserProfile {
                username: _,
                name: _,
                email: _,
                shell,
                editor,
                ssh,
            } = home;
            match shell {
                Some(config::Shell::Bash) => {
                    ret.extend([self.env.calculate_managed_dotfile(".bashrc")])
                }
                None => {}
            }
            match editor {
                Some(config::Editor::Helix) => {
                    self.env
                        .calculate_managed_config_dir("helix", &mut ret, Filter::HELIX)?
                }
                None => {}
            }
            match ssh {
                Some(config::Ssh::OpenSSH {
                    agent: config::SshAgent::SystemD,
                    keys,
                }) => {
                    ret.push(
                        self.env
                            .calculate_managed_config_file("systemd/user/ssh-agent.service"),
                    );
                    debug_assert!(keys.is_empty(), "TODO: {keys:?}");
                }
                None => {}
            }
        }
        Ok(ret)
    }

    pub fn calculate_status(&self) -> Result<Status, Error> {
        Status::calculate(self)
    }
}
