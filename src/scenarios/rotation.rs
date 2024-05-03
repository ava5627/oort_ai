use crate::utils::turn_to;
use oort_api::prelude::*;
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
        let angle = lead_target(target(), vec2(0.0, 0.0), vec2(0.0, 0.0), 1000.0);
        turn_to(angle);
        let error = angle_diff(angle, heading());
        if error.abs() < 0.2 {
            fire(0);
        }
        accelerate(vec2(100.0, 0.0).rotate(angle));
    }
}
fn lead_target(
    target_position: Vec2,
    target_velocity: Vec2,
    acceleration: Vec2,
    bullet_speed: f64,
) -> f64 {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    let time_to_target = dp.length() / bullet_speed;
    let mut future_position =
        dp + dv * time_to_target + acceleration * time_to_target.powf(2.0) / 2.0;
    for _ in 0..100 {
        let time_to_target = future_position.length() / bullet_speed;
        let new_future_position =
            dp + dv * time_to_target + acceleration * time_to_target.powf(2.0) / 2.0;
        let delta = new_future_position - future_position;
        if delta.length() < 1e-3 {
            break;
        }
        future_position = new_future_position;
    }
    let color = 0x00ff00;
    draw_polygon(future_position + position(), 10.0, 4, 0.0, color);
    (future_position).angle()
}
