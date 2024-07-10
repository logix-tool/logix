use crate::{env::Env, error::Error, status::Status};
use config::{ConfigDir, Filter, Package};
use logix_type::LogixLoader;
use logix_vfs::RelFs;
use managed_file::ManagedFile;
use managed_files::ManagedFiles;

pub mod based_path;
pub mod config;
pub mod env;
pub mod error;
pub mod managed_file;
pub mod managed_files;
pub mod status;
mod walk_dir;

/// This is the root of a logix session. Most functionality will start
/// by creating an instance of this struct
pub struct Logix {
    env: Env,
    config: config::Logix,
}

impl Logix {
    /// Load the logix instance from the specified environment. This includes loading the config files.
    pub fn load(env: Env) -> Result<Self, Error> {
        let mut loader = LogixLoader::new(RelFs::new(env.logix_root()));
        Ok(Self {
            env,
            config: loader.load_file("root.logix")?,
        })
    }

    /// Retrieve the raw config
    pub fn config(&self) -> &config::Logix {
        &self.config
    }

    /// Returns a list of files managed by logix
    pub fn calculate_managed_files(&self) -> Result<Vec<ManagedFile>, Error> {
        let config::Logix { home } = &self.config;
        let mut ret = ManagedFiles::new(&self.env);
        {
            let config::UserProfile {
                username: _,
                name: _,
                email: _,
                shell,
                editor: _,
                ssh,
                packages,
            } = home;
            match shell {
                Some(config::Shell::Bash) => ret.add_dotfile(".bashrc")?,
                None => {}
            }
            match ssh {
                Some(config::Ssh::OpenSSH {
                    agent: config::SshAgent::SystemD,
                    keys,
                }) => {
                    ret.add_config_file("systemd/user/ssh-agent.service")?;
                    debug_assert!(keys.is_empty(), "TODO: {keys:?}");
                }
                None => {}
            }
            for (pname, p) in packages {
                match p {
                    Package::Custom {
                        source: _,
                        config_dir,
                    } => match config_dir {
                        ConfigDir::User {
                            package_name,
                            filter,
                        } => ret.add_dir(
                            &self
                                .env
                                .user_config()
                                .make_shadowed_subdir(package_name.as_ref().unwrap_or(pname))?,
                            filter.as_ref().unwrap_or(Filter::EMPTY),
                        )?,
                    },
                }
            }
        }
        Ok(ret.finalize())
    }

    /// Calculate the status of all the files managed by logix
    pub fn calculate_status(&self) -> Result<Status, Error> {
        Status::calculate(self)
    }
}
