use oort_api::prelude::*;

use crate::target::Target;
use crate::utils::{turn_to, VecUtils};
pub struct Ship {
    target: Target,
}
impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        if let Some((pos, vel)) = recieve_pos_vel() {
            Ship {
                target: Target::new(
                    pos,
                    vel,
                    Class::Fighter, // Assuming the class is Fighter for this example
                ),
            }
        } else {
            Ship {
                target: Target::new(Vec2::zero(), Vec2::zero(), Class::Fighter),
            }
        }
    }
    pub fn tick(&mut self) {
        set_radio_channel(2);
        if let Some((pos, vel)) = recieve_pos_vel() {
            self.target.update(pos, vel);
        }
        self.target.tick(0);
        self.target.draw_path();
        let prediction = self.target.lead(0);
        let angle = prediction.angle();
        turn_to(angle);
        let miss_by = angle_diff(angle, heading()) * prediction.length();
        if miss_by < 10.0 {
            fire(0);
        }
        accelerate(Vec2::angle_length(angle, max_forward_acceleration()));
    }
}

pub fn recieve_pos_vel() -> Option<(Vec2, Vec2)> {
    if let Some(data) = receive() {
        let pos = vec2(data[0], data[1]);
        let vel = vec2(data[2], data[3]);
        return Some((pos, vel));
    }
    None
}
