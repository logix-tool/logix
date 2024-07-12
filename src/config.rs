use std::{path::Path, sync::Arc};

use logix_type::{
    types::{ExecutablePath, Map, ShortStr, ValidPath},
    LogixType,
};

/// Supported shells
#[derive(Debug, LogixType, Clone, Copy)]
pub enum Shell {
    /// the Bourne Again SHell
    Bash,
}

/// Supported ways of managing the ssh agent
#[derive(Debug, LogixType)]
pub enum SshAgent {
    /// Run `ssh-agent` as a systemd user service
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

/// A relative path filter
#[derive(Debug, LogixType, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Filter {
    /// Ignore all paths that starts with exactly this. It will match entire
    /// components, so `"hello"` will not match `"hello.txt"`, but it will
    /// match `"hello/world.txt"`
    pub ignore_starts_with: Vec<ShortStr>,
}

impl Filter {
    pub const EMPTY: &'static Filter = &Filter {
        ignore_starts_with: Vec::new(),
    };

    /// Check if the specified relatiove path should be included
    pub fn should_include(&self, path: &Path) -> bool {
        self.ignore_starts_with.iter().all(|v| !path.starts_with(v))
    }
}

/// Points to the config of a [Package]
#[derive(Debug, LogixType, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfigDir {
    /// The config is under the standard user directory, such as
    /// `~/.config/<package_name>` and all files that fit the `filter`
    /// will be managed.
    User {
        package_name: Option<ShortStr>,
        filter: Option<Filter>,
    },
}

/// A package of various types that is managed by logix
#[derive(Debug, LogixType, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Package {
    /// The package is a rust crate installed using cargo
    RustCrate {
        /// Name of the crate as recognized by cargo and crates.io, defaults to the package name
        crate_name: Option<ShortStr>,
        /// Specified the config files that needs to be managed by this package
        config_dir: ConfigDir,
    },
    /// A custom package which will be downloaded and built from source. It
    /// will also be kept up to date by logix.
    Custom {
        /// The source of the package such as the repository it can be built from
        source: Source,
        /// Where to store the downloaded sources, defaults to a predictable directory
        /// under `./cache/logix` or similar (depending on platform)
        local_dir: Option<ValidPath>,
        // TODO: install: Command,
        /// Specified the config files that needs to be managed by this package
        config_dir: ConfigDir,
    },
}

/// Points to the source of a [Package] and may be used to look for new versions
#[derive(Debug, LogixType, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    GitHub { owner: ShortStr, repo: ShortStr },
}

#[derive(Debug, LogixType)]
pub struct UserProfile {
    pub username: ShortStr,
    pub name: ShortStr,
    pub email: ShortStr,
    /// The default shell to use. This will also affect which config files are managed
    pub shell: Option<Shell>,
    /// The default editor to use for various operations such as edition the git commit message
    pub editor: Option<ExecutablePath>,
    pub ssh: Option<Ssh>,
    /// Packages installed to the users home directory, such as to `local/.bin`
    pub packages: Map<Package, Arc<str>>,
}

/// The root of the logix config
#[derive(Debug, LogixType)]
pub struct Logix {
    /// Options for the user that owns this logix config
    pub home: UserProfile,
}
