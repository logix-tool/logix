use logix_type::{LogixType, Map};

#[derive(Debug, LogixType)]
pub enum Shell {
    Bash,
}

#[derive(Debug, LogixType)]
pub enum Editor {
    Helix,
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
pub struct UserProfile {
    pub username: String,
    pub name: String,
    pub email: String,
    pub shell: Option<Shell>,
    pub editor: Option<Editor>,
    pub ssh: Option<Ssh>,
}

#[derive(Debug, LogixType)]
pub struct Logix {
    pub home: UserProfile,
}
