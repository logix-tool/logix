use std::path::Path;

use crate::{
    config::Filter,
    env::{Env, ShadowedDir},
    error::Error,
    managed_file::{LocalFile, ManagedFile, Owner},
    walk_dir::{walk_dirs, WalkEntry},
};

pub struct ManagedFiles<'env> {
    env: &'env Env,
    files: Vec<ManagedFile>,
}

impl<'env> ManagedFiles<'env> {
    pub fn new(env: &'env Env) -> Self {
        Self {
            env,
            files: Vec::new(),
        }
    }

    /// Add a [ManagedFile] to the list of files
    pub fn add_file(&mut self, file: ManagedFile) {
        self.files.push(file);
    }

    /// Add the files from the given [ShadowedDirectory] using the specified [Filter]
    pub fn add_dir(
        &mut self,
        owner: &Owner,
        dir: &ShadowedDir,
        local_filter: &Filter,
    ) -> Result<(), Error> {
        walk_dirs(dir, local_filter, |entry| match entry {
            WalkEntry::Local(rel_path) | WalkEntry::Both(rel_path) | WalkEntry::Logix(rel_path) => {
                self.add_local_file(owner, dir, rel_path)
            }
        })
    }

    pub fn add_local_file(
        &mut self,
        owner: &Owner,
        dir: &ShadowedDir,
        rel_path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        self.add_file(ManagedFile::Local(
            owner.clone(),
            dir.make_local_file(rel_path)?,
        ));
        Ok(())
    }

    pub fn add_config_file(
        &mut self,
        owner: &Owner,
        rel_path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        self.add_local_file(owner, self.env.user_config(), rel_path)
    }

    pub fn add_dotfile(&mut self, owner: &Owner, rel_path: impl AsRef<Path>) -> Result<(), Error> {
        let rel_path = rel_path.as_ref();
        let stripped_name = rel_path
            .file_name()
            .ok_or_else(|| Error::GetFileName(rel_path.into()))
            .and_then(|name| {
                name.to_str()
                    .ok_or_else(|| Error::FileNameToStr(rel_path.into()))
            })
            .and_then(|name| {
                name.strip_prefix('.')
                    .ok_or_else(|| Error::FileNameNotDotfile(rel_path.into()))
            })?;

        self.add_file(ManagedFile::Local(
            owner.clone(),
            LocalFile {
                local: self.env.dotfiles().local_path().join(rel_path)?,
                logix: self
                    .env
                    .dotfiles()
                    .logix_path()
                    .join(rel_path.with_file_name(stripped_name))?,
            },
        ));
        Ok(())
    }

    pub(crate) fn finalize(self) -> Vec<ManagedFile> {
        self.files
    }
}
