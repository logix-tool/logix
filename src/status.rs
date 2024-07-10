use crate::{error::Error, managed_file::CmpRes, ManagedFile};

/// Contains the status for a set of managed files
#[derive(Debug)]
pub struct Status {
    modified: Vec<ManagedFile>,
    missing: Vec<ManagedFile>,
    local_added: Vec<ManagedFile>,
    logix_added: Vec<ManagedFile>,
    up_to_date: Vec<ManagedFile>,
}

impl Status {
    pub(crate) fn calculate(logix: &crate::Logix) -> Result<Status, Error> {
        let mut ret = Self {
            modified: Vec::new(),
            missing: Vec::new(),
            local_added: Vec::new(),
            logix_added: Vec::new(),
            up_to_date: Vec::new(),
        };
        for entry in logix.calculate_managed_files()? {
            match entry.compare_files()? {
                CmpRes::UpToDate => ret.up_to_date.push(entry),
                CmpRes::Missing => ret.missing.push(entry),
                CmpRes::LocalAdded => ret.local_added.push(entry),
                CmpRes::LogixAdded => ret.logix_added.push(entry),
                CmpRes::Modified => ret.modified.push(entry),
            }
        }
        Ok(ret)
    }

    /// Return a slice with all the modified files
    pub fn modified(&self) -> &[ManagedFile] {
        &self.modified
    }

    /// Return a slice with all the files that logix expects, but who is missing from both the
    /// logix directory and the user directory
    pub fn missing(&self) -> &[ManagedFile] {
        &self.missing
    }

    /// Return a slice with all the files that logix does not manage, yet are in the user directory
    pub fn local_added(&self) -> &[ManagedFile] {
        &self.local_added
    }

    /// Return a slice with all the files that logix expects, but is missing from the user directory
    pub fn logix_added(&self) -> &[ManagedFile] {
        &self.logix_added
    }

    /// Return a slice with all the files that are up to date
    pub fn up_to_date(&self) -> &[ManagedFile] {
        &self.up_to_date
    }
}
