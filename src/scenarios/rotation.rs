use crate::utils::{boost, turn_to};
use oort_api::prelude::*;

pub struct Ship {
    boost_ticks: Option<usize>,
}

impl Ship {
    pub fn new() -> Ship {
        Ship { boost_ticks: None }
    }

    pub fn tick(&mut self) {
        let dp = target() - position();
        let angle = dp.angle();
        turn_to(angle);
        accelerate(target() - position());
        if angle_diff(heading(), angle).abs() < PI / 10.0 {
            boost(
                angle_diff(heading(), angle).abs() < PI / 5.0,
                &mut self.boost_ticks,
            );
            fire(0);
        }
    }
}

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

