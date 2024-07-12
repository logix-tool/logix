use crate::{env::Env, error::Error};
use config::{ConfigDir, Filter, Package};
use logix_type::LogixLoader;
use logix_vfs::{MemFs, RelFs};
use managed_file::{FileStatus, ManagedFile};
use managed_files::ManagedFiles;
use std::fmt::Write as _;

pub mod based_path;
pub mod config;
pub mod env;
pub mod error;
pub mod managed_file;
pub mod managed_files;
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
    pub fn calculate_status(
        &self,
    ) -> Result<impl ExactSizeIterator<Item = (FileStatus, ManagedFile)>, Error> {
        Ok(self
            .calculate_managed_files()?
            .into_iter()
            .map(|file| (file.calculate_status(), file)))
    }
}

pub struct LogixConfigGenerator<'a> {
    pub username: &'a str,
    pub name: &'a str,
    pub email: &'a str,
    pub shell: config::Shell,
    pub editor: &'a str,
}

impl<'a> LogixConfigGenerator<'a> {
    pub fn generate(&self) -> Result<String, Error> {
        let Self {
            username,
            name,
            email,
            shell,
            editor,
        } = *self;

        let mut ret = String::with_capacity(1024);
        writeln!(ret, "Logix {{").unwrap();
        writeln!(ret, "  home: UserProfile {{").unwrap();
        writeln!(ret, "    username: {username:?}").unwrap();
        writeln!(ret, "    name: {name:?}").unwrap();
        writeln!(ret, "    email: {email:?}").unwrap();
        writeln!(ret, "    shell: {shell:?}").unwrap();
        writeln!(ret, "    editor: {editor:?}").unwrap();
        writeln!(ret, "    packages: {{").unwrap();
        writeln!(ret, "      // TODO: Add packages to manage").unwrap();
        writeln!(ret, "    }}").unwrap();
        writeln!(ret, "  }}").unwrap();
        writeln!(ret, "}}").unwrap();

        let mut fs = MemFs::default();
        fs.set_file("root.logix", ret.as_bytes(), true).unwrap(); // NOTE: Can't fail

        LogixLoader::new(fs)
            .load_file::<config::Logix>("root.logix")
            .map_err(|e| Error::InvalidGeneratedConfig("root.logix", format!("{e:?}")))?;

        Ok(ret)
    }
}
