// Tutorial: Acceleration
// Fly through the target circle.
use oort_api::prelude::*;

use crate::{scenarios::missiles::turn_to, utils::{best_acceleration, boost, max_accelerate}};

pub struct Ship {}

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        // Hint: uncomment me
        debug!("{}", current_tick());
        boost(true, &mut None);
        let ma = best_acceleration(heading());
        turn_to(-ma.angle());
        max_accelerate(ma);
    }
}
