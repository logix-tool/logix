use std::path::{Path, PathBuf};

use directories::{BaseDirs, ProjectDirs, UserDirs};

use crate::{error::Error, LocalFile, ManagedFile};

pub(crate) struct Filter {
    pub ignore_starts_with: &'static [&'static str],
}

impl Filter {
    pub const EMPTY: &'static Filter = &Filter {
        ignore_starts_with: &[],
    };
    pub const HELIX: &'static Filter = &Filter {
        ignore_starts_with: &["runtime/"],
    };

    fn should_include(&self, path: &Path) -> bool {
        self.ignore_starts_with.iter().all(|v| !path.starts_with(v))
    }
}

pub struct Env {
    base_dirs: BaseDirs,
    _user_dirs: UserDirs,
    logix_dir: ProjectDirs,
    logix_config_files_dir: PathBuf,
    logix_dotfiles_dir: PathBuf,
}

impl Env {
    pub fn init() -> Result<Self, Error> {
        let logix_dir = ProjectDirs::from("com", "logix-tool", "logix").ok_or(Error::LocateHome)?;
        let logix_config_files_dir = logix_dir.config_dir().join("config");
        let logix_dotfiles_dir = logix_dir.config_dir().join("dotfiles");
        Ok(Self {
            base_dirs: BaseDirs::new().ok_or(Error::LocateHome)?,
            _user_dirs: UserDirs::new().ok_or(Error::LocateHome)?,
            logix_dir,
            logix_config_files_dir,
            logix_dotfiles_dir,
        })
    }

    pub(crate) fn calculate_managed_config_dir(
        &self,
        dir_name: &str,
        res: &mut Vec<ManagedFile>,
        local_filter: &Filter,
    ) -> Result<(), Error> {
        let local_dir = self.base_dirs.config_dir().join(dir_name);
        let logix_dir = self.logix_config_files_dir.join(dir_name);
        let mut local_it = walk_dir(&local_dir, local_filter);
        let mut logix_it = walk_dir(&logix_dir, Filter::EMPTY);

        let by_local_path = |local: PathBuf| -> Result<_, Error> {
            let logix = logix_dir.join(
                local
                    .strip_prefix(&local_dir)
                    .map_err(|_| Error::StripPrefixFailed)?,
            );
            Ok(ManagedFile::Local(LocalFile { local, logix }))
        };
        let by_logix_path = |logix: PathBuf| -> Result<_, Error> {
            let local = local_dir.join(
                logix
                    .strip_prefix(&logix_dir)
                    .map_err(|_| Error::StripPrefixFailed)?,
            );
            Ok(ManagedFile::Local(LocalFile { local, logix }))
        };

        let mut cur_local = local_it.next().transpose()?;
        let mut cur_logix = logix_it.next().transpose()?;

        loop {
            match (cur_local.take(), cur_logix.take()) {
                (Some(local), Some(logix)) => {
                    let local_rel = local
                        .strip_prefix(&local_dir)
                        .map_err(|_| Error::StripPrefixFailed)?;
                    let logix_rel = logix
                        .strip_prefix(&logix_dir)
                        .map_err(|_| Error::StripPrefixFailed)?;
                    match local_rel.cmp(logix_rel) {
                        std::cmp::Ordering::Less => {
                            res.push(by_local_path(local)?);
                            cur_local = local_it.next().transpose()?;
                            cur_logix = Some(logix);
                        }
                        std::cmp::Ordering::Equal => {
                            res.push(ManagedFile::Local(LocalFile { local, logix }));
                            cur_local = local_it.next().transpose()?;
                            cur_logix = logix_it.next().transpose()?;
                        }
                        std::cmp::Ordering::Greater => {
                            res.push(by_logix_path(logix)?);
                            cur_logix = logix_it.next().transpose()?;
                            cur_local = Some(local);
                        }
                    }
                }
                (Some(local), None) => {
                    res.push(by_local_path(local)?);
                    cur_local = local_it.next().transpose()?;
                }
                (None, Some(logix)) => {
                    res.push(by_logix_path(logix)?);
                    cur_logix = logix_it.next().transpose()?;
                }
                (None, None) => return Ok(()),
            }
        }
    }

    pub(crate) fn calculate_managed_config_file(&self, sub_path: &str) -> ManagedFile {
        ManagedFile::Local(LocalFile {
            local: self.base_dirs.config_dir().join(sub_path),
            logix: self.logix_config_files_dir.join(sub_path),
        })
    }

    pub(crate) fn calculate_managed_dotfile(&self, name: &str) -> ManagedFile {
        ManagedFile::Local(LocalFile {
            local: self.base_dirs.home_dir().join(name),
            logix: self.logix_dotfiles_dir.join(name),
        })
    }

    pub(crate) fn logix_config_dir(&self) -> &Path {
        self.logix_dir.config_dir()
    }
}

fn walk_dir<'a>(
    path: &'a Path,
    filter: &'a Filter,
) -> impl Iterator<Item = Result<PathBuf, Error>> + 'a {
    path.exists()
        .then(move || {
            walkdir::WalkDir::new(path)
                .follow_links(false)
                .sort_by_file_name()
                .into_iter()
                .filter_entry(move |v| filter.should_include(v.path().strip_prefix(path).unwrap()))
                .filter_map(|v| match v {
                    Ok(v) => {
                        if v.file_type().is_dir() {
                            None
                        } else {
                            Some(Ok(v.into_path()))
                        }
                    }
                    Err(e) => Some(Err(Error::WalkDir(e))),
                })
        })
        .into_iter()
        .flatten()
}
