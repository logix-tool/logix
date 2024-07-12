use std::path::Path;

use crate::{config::Filter, env::ShadowedDir, error::Error};

pub enum WalkEntry<'a> {
    Local(&'a Path),
    Both(&'a Path),
    Logix(&'a Path),
}

pub fn walk_dirs(
    base: &ShadowedDir,
    local_filter: &Filter,
    mut on_file: impl FnMut(WalkEntry) -> Result<(), Error>,
) -> Result<(), Error> {
    let walk_iter = |path: &Path| {
        path.exists()
            .then(|| {
                walkdir::WalkDir::new(path)
                    .follow_links(false)
                    .sort_by_file_name()
            })
            .into_iter()
            .flatten()
            .filter(|v| {
                if let Ok(v) = v {
                    // Ignore directories
                    !v.path().is_dir()
                } else {
                    true
                }
            })
    };

    let mut local_it = walk_iter(base.local_path());
    let mut logix_it = walk_iter(base.logix_path());

    let mut next_local = || local_it.next().transpose().map_err(Error::WalkDir);
    let mut next_logix = || logix_it.next().transpose().map_err(Error::WalkDir);

    let mut cur_local = next_local()?;
    let mut cur_logix = next_logix()?;

    loop {
        match (cur_local, cur_logix) {
            (Some(local), Some(logix)) => {
                let local_path = local.path().strip_prefix(base.local_path()).unwrap();
                if !local_filter.should_include(local_path) {
                    cur_local = next_local()?;
                    cur_logix = Some(logix);
                    continue;
                }

                let logix_path = logix.path().strip_prefix(base.logix_path()).unwrap();

                match local_path.cmp(logix_path) {
                    std::cmp::Ordering::Less => {
                        on_file(WalkEntry::Local(local_path))?;
                        cur_local = next_local()?;
                        cur_logix = Some(logix);
                    }
                    std::cmp::Ordering::Equal => {
                        on_file(WalkEntry::Both(local_path))?;
                        cur_local = next_local()?;
                        cur_logix = next_logix()?;
                    }
                    std::cmp::Ordering::Greater => {
                        on_file(WalkEntry::Logix(logix_path))?;
                        cur_local = Some(local);
                        cur_logix = next_logix()?;
                    }
                }
            }
            (Some(local), None) => {
                let local_path = local.path().strip_prefix(base.local_path()).unwrap();
                if !local_filter.should_include(local_path) {
                    cur_local = next_local()?;
                    cur_logix = None;
                    continue;
                }

                on_file(WalkEntry::Local(local_path))?;

                cur_local = next_local()?;
                cur_logix = None;
            }
            (None, Some(logix)) => {
                let logix_path = logix.path().strip_prefix(base.logix_path()).unwrap();
                on_file(WalkEntry::Logix(logix_path))?;

                cur_local = None;
                cur_logix = next_logix()?;
            }
            (None, None) => return Ok(()),
        }
    }
}
