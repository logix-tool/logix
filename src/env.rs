use std::path::Path;

use logix_type::types::FullPath;

use crate::{based_path::BasedPath, error::Error, managed_file::LocalFile};

/// Represents a local directory that has a corresponding directory in the logix config
#[derive(Debug)]
pub struct ShadowedDir {
    local: BasedPath,
    logix: BasedPath,
}

impl ShadowedDir {
    pub fn make_local_file(&self, rel_path: impl AsRef<Path>) -> Result<LocalFile, Error> {
        let rel_path = rel_path.as_ref();
        Ok(LocalFile {
            local: self.local.join(rel_path)?,
            logix: self.logix.join(rel_path)?,
        })
    }

    pub fn make_shadowed_subdir(&self, rel_path: impl AsRef<Path>) -> Result<Self, Error> {
        let rel_path = rel_path.as_ref();
        Ok(Self {
            local: self.local.join(rel_path)?,
            logix: self.logix.join(rel_path)?,
        })
    }

    pub fn local_path(&self) -> &BasedPath {
        &self.local
    }

    pub fn logix_path(&self) -> &BasedPath {
        &self.logix
    }
}

/// Builder used to specify the environment to use when initializing [Env], created by calling [Env::builder]
pub struct EnvBuilder {
    home_dir: Option<FullPath>,
}

impl EnvBuilder {
    /// Fill in any missing data from the OS environment
    pub fn init_from_os_env(&mut self) -> Result<&mut Self, Error> {
        if self.home_dir.is_none() {
            self.home_dir = Some(
                FullPath::try_from(home::home_dir().ok_or(Error::LocateDir("user home"))?)
                    .map_err(|e| Error::InvalidDir("user home", e))?,
            );
        }

        Ok(self)
    }

    /// Try to build the [Env], fails if not all required options are set
    pub fn build(&mut self) -> Result<Env, Error> {
        let user_dir = self
            .home_dir
            .take()
            .map(BasedPath::new)
            .ok_or(Error::MissingDir("EnvBuilder::home_dir"))?;

        #[cfg(unix)]
        let user_config_dir = user_dir.join(".config")?;

        // TODO: Calculate the config directory on other platforms

        let logix_root = user_config_dir.join("logix")?.rebased();

        let user_cache = user_dir.join(".cache")?;

        Ok(Env {
            user_config: ShadowedDir {
                local: user_config_dir.clone(),
                logix: logix_root.join("config")?,
            },
            dotfiles: ShadowedDir {
                local: user_dir,
                logix: logix_root.join("dotfiles")?,
            },

            logix_root,

            user_cache,
        })
    }

    pub fn home_dir(&mut self, path: FullPath) -> &mut Self {
        self.home_dir = Some(path);
        self
    }
}

/// Contains a pre-calculated version of the environment such as various directories.
pub struct Env {
    /// ~/.config <-> ~/.config/logix/config
    user_config: ShadowedDir,
    /// ~/ <-> ~/.config/logix/dotfiles
    dotfiles: ShadowedDir,

    /// ~/.config/logix
    logix_root: BasedPath,

    /// ~/.cache
    user_cache: BasedPath,
}

impl Env {
    pub fn builder() -> EnvBuilder {
        EnvBuilder { home_dir: None }
    }

    /// Create an [Env] instance using the OS environment
    pub fn init() -> Result<Self, Error> {
        Self::builder().init_from_os_env()?.build()
    }

    /// Returns the users config directory, such as `~/.config` and the corresponding
    /// logix directory such as `~/.config/helix/config`
    pub fn user_config(&self) -> &ShadowedDir {
        &self.user_config
    }

    /// Returns the root of the logix config directory such as `~/.config/logix`
    pub fn logix_root(&self) -> &BasedPath {
        &self.logix_root
    }

    /// Returns the directory containing dotfiles such as `~` and the corresponding
    /// logix directory such as `~/.config/helix/dotfiles`
    pub fn dotfiles(&self) -> &ShadowedDir {
        &self.dotfiles
    }

    /// Returns the cache directory of the current user such as `~/.cache`
    pub fn user_cache(&self) -> &BasedPath {
        &self.user_cache
    }
}
