use logix::{env::Env, error::Error, Logix};

use crate::SharedArgs;

use super::theme::Theme;

pub struct Context {
    pub logix: Logix,
    pub theme: Theme,
    pub args: SharedArgs,
}

impl Context {
    pub fn load(theme: Theme, args: SharedArgs) -> Result<Self, Error> {
        Ok(Self {
            logix: Logix::load(Env::init()?)?,
            theme,
            args,
        })
    }

    pub fn write_fmt(&self, args: std::fmt::Arguments) {
        print!("{args}");
    }
}
