use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    // Uncomment me, then press Ctrl-Enter (Cmd-Enter on Mac) to upload the code.
    pub fn tick(&mut self) {
        fire(0);
    }
}

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}
