use oort_api::prelude::*;

use crate::utils::turn_to;
const BULLET_SPEED: f64 = 1000.0; // m/s
pub struct Ship {
    target_last_velocity: Vec2,
}
impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            target_last_velocity: vec2(0.0, 0.0),
        }
    }
    pub fn tick(&mut self) {
        set_radio_channel(2);
        if receive().is_none() {
            debug!("no message received");
            return;
        }
        activate_ability(Ability::Boost);
        let msg = receive().unwrap();
        let contact_position = vec2(msg[0], msg[1]);
        let contact_velocity = vec2(msg[2], msg[3]);
        let aim_point = self.lead_target(contact_position, contact_velocity);
        let target_heading = aim_point.angle();
        turn_to(target_heading);
        accelerate(vec2(100.0, 0.0).rotate(target_heading));
        fire(0);
    }
    pub fn lead_target(&mut self, target_position: Vec2, target_velocity: Vec2) -> Vec2 {
        let dp = target_position - position();
        let dv = target_velocity - velocity();
        let acc = (self.target_last_velocity - target_velocity) / TICK_LENGTH;
        self.target_last_velocity = target_velocity;
        let mut time_to_target = dp.length() / BULLET_SPEED;
        let mut future_position = dp + dv * time_to_target + 0.5 * acc * time_to_target.powi(2);
        let mut delta = future_position;
        for _ in 0..100 {
            time_to_target = future_position.length() / BULLET_SPEED;
            let new_future_position = dp + dv * time_to_target + 0.5 * acc * time_to_target.powi(2);
            delta = new_future_position - future_position;
            future_position = new_future_position;
            if delta.length() < 1e-3 {
                break;
            }
        }
        debug!("delta: {}", delta.length());
        debug!("heading: {}", heading());
        debug!("future position: {}", future_position.angle());
        draw_polygon(future_position + position(), 10.0, 4, 0.0, 0xffffff);
        draw_line(position(), future_position + position(), 0xffffff);
        let d = future_position.length();
        let v = vec2(d, 0.0).rotate(heading());
        draw_line(position(), position() + v, 0xff0000);
        future_position
    }
}
