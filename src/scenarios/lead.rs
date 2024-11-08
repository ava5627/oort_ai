use oort_api::prelude::*;

use crate::target::Target;
use crate::utils::{lead_target, turn_to, VecUtils};
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
        let target = Target::new(target(), target_velocity(), Class::Fighter);
        let predicted_position = lead_target(&target, 0);
        draw_triangle(predicted_position, 10.0, 0xff0000);
        draw_line(position(), predicted_position, 0xff0000);
        draw_line(
            position(),
            position() + Vec2::angle_length(heading(), predicted_position.distance(position())),
            0x00ff00,
        );
        let angle = predicted_position.angle();
        turn_to(angle);
        if angle_diff(heading(), angle).abs() < 0.01 {
            fire(0);
        }
    }
}
