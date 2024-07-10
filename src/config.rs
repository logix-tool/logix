use std::path::Path;

use logix_type::{
    types::{ExecutablePath, Map, ShortStr},
    LogixType,
};

#[derive(Debug, LogixType)]
pub enum Shell {
    Bash,
}

#[derive(Debug, LogixType)]
pub enum SshAgent {
    SystemD,
}

#[derive(Debug, LogixType)]
pub enum Ssh {
    OpenSSH { agent: SshAgent, keys: Map<String> },
}

#[derive(Debug, LogixType)]
pub enum Command {
    ShellCommand { command: String },
}

#[derive(Debug, LogixType)]
pub struct Filter {
    pub ignore_starts_with: Vec<ShortStr>,
}

impl Filter {
    pub const EMPTY: &'static Filter = &Filter {
        ignore_starts_with: Vec::new(),
    };

    pub fn should_include(&self, path: &Path) -> bool {
        self.ignore_starts_with.iter().all(|v| !path.starts_with(v))
    }
}

#[derive(Debug, LogixType)]
pub enum ConfigDir {
    User {
        package_name: Option<ShortStr>,
        filter: Option<Filter>,
    },
}

#[derive(Debug, LogixType)]
pub enum Package {
    Custom {
        /// The source of the package such as the repository it can be built from
        source: Source,
        // TODO: local_dir: ValidPath,
        // TODO: install: Command,
        config_dir: ConfigDir,
    },
}

#[derive(Debug, LogixType)]
pub enum Source {
    GitHub { owner: ShortStr, repo: ShortStr },
}

#[derive(Debug, LogixType)]
pub struct UserProfile {
    pub username: ShortStr,
    pub name: ShortStr,
    pub email: ShortStr,
    pub shell: Option<Shell>,
    pub editor: Option<ExecutablePath>,
    pub ssh: Option<Ssh>,
    pub packages: Map<Package>,
}

#[derive(Debug, LogixType)]
pub struct Logix {
    pub home: UserProfile,
}
