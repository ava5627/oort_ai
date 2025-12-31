use oort_api::prelude::*;

use crate::utils::{turn_to, VecUtils};

pub struct Test {
    target_heading: f64,
    shot_time: u32,
    start_time: u32,
}
impl Default for Test {
    fn default() -> Self {
        Self::new()
    }
}

impl Test {
    pub fn new() -> Test {
        debug!("spawn frigate team 0 position (100, 0) heading 0");
        Test {
            target_heading: -5.5 * PI / 30.0,
            shot_time: 0,
            start_time: current_tick(),
        }
    }
    pub fn tick(&mut self) {
        draw_line(
            position(),
            position() + Vec2::angle_length(self.target_heading, 1000.0),
            0x00ff00,
        );
        draw_line(
            position(),
            position() + Vec2::angle_length(heading(), 1000.0),
            0xff0000,
        );
        turn_to(self.target_heading);
        let curr_error = angle_diff(self.target_heading, heading());
        debug!("curr_error {:?}", curr_error);
        if curr_error.abs() < 0.01 && reload_ticks(0) == 0 {
            fire(0);
            self.target_heading = -self.target_heading;
            self.shot_time = current_tick() - self.start_time;
            self.start_time = current_tick();
        }
        debug!("shot_time {:?}", self.shot_time);
    }

}
