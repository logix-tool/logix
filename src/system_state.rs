use crate::{error::Error, helpers::cargo::CargoState};

pub struct SystemState {
    pub cargo: CargoState,
}

impl SystemState {
    pub fn init() -> Result<Self, Error> {
        Ok(Self {
            cargo: CargoState::init()?,
        })
    }

    pub fn refresh_state(&mut self) -> Result<(), Error> {
        let Self { cargo } = self;
        cargo.refresh_state()?;
        Ok(())
    }
}
