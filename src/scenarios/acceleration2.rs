use crate::{target::Target, utils::{boost, seek}};
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
        boost(angle_diff(heading(), angle).abs() < PI / 5.0, &mut self.boost_ticks);
        let target = Target::new(target(), target_velocity(), Class::Unknown);
        seek(&target);
    }
}

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}
