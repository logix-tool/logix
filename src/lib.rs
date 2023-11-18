use logix_type::{types::Data, LogixType, Map};

mod env;
mod error;

pub use crate::{
    env::Env,
    error::{Error, Result},
};

#[derive(Debug, LogixType)]
pub enum Shell {
    Bash { bashrc: Data<String> },
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
pub enum Profile {
    User {
        name: String,
        email: String,
        shell: Option<Shell>,
        ssh: Option<Ssh>,
    },
}

#[derive(Debug, LogixType)]
pub struct Logix {
    pub profiles: Map<Profile>,
}
