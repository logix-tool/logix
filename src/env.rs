use std::path::Path;

#[derive(Debug)]
pub struct Env<'a> {
    pub config_root: &'a Path,
}
