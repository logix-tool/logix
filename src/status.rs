use crate::{error::Error, CmpRes, ManagedFile};

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

    pub fn modified(&self) -> &[ManagedFile] {
        &self.modified
    }

    pub fn missing(&self) -> &[ManagedFile] {
        &self.missing
    }

    pub fn local_added(&self) -> &[ManagedFile] {
        &self.local_added
    }

    pub fn logix_added(&self) -> &[ManagedFile] {
        &self.logix_added
    }

    pub fn up_to_date(&self) -> &[ManagedFile] {
        &self.up_to_date
    }
}
