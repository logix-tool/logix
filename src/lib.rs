use logix_type::{LogixType, Map};

#[derive(LogixType)]
pub enum Shell {
    Bash { bashrc: String },
}

#[derive(LogixType)]
pub enum Profile {
    User {
        name: String,
        email: String,
        shell: Shell,
    },
}

#[derive(LogixType)]
pub struct Logix {
    pub profiles: Map<Profile>,
}
