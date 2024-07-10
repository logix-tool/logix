use std::path::{Path, PathBuf};

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

pub struct Env {
    /// ~/.config <-> ~/.config/logix/config
    user_config: ShadowedDir,
    /// ~/ <-> ~/.config/logix/dotfiles
    dotfiles: ShadowedDir,

    /// ~/.config/logix
    logix_root: BasedPath,
}

impl Env {
    /// Create a new environment, using the resolved system paths for the current user
    pub fn init() -> Result<Self, Error> {
        fn get_dir(p: Option<PathBuf>) -> Result<BasedPath, Error> {
            FullPath::try_from(p.ok_or(Error::LocateDir("user home"))?)
                .map(BasedPath::new)
                .map_err(|e| Error::InvalidDir("user home", e))
        }

        let user_dir = get_dir(dirs::home_dir())?;
        let user_config_dir = get_dir(dirs::config_dir())?;
        let logix_root = user_config_dir.join("logix")?;

        Ok(Self {
            user_config: ShadowedDir {
                local: user_config_dir.clone(),
                logix: logix_root.join("config")?,
            },
            dotfiles: ShadowedDir {
                local: user_dir,
                logix: user_config_dir.join("logix/dotfiles")?,
            },

            logix_root,
        })
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
}
