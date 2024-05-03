use crate::{utils::angle_at_distance, vec_utils::VecUtils};
use oort_api::prelude::*;
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub position: Vec2,
    pub velocity: Vec2,
    pub last_velocity: Vec2,
    pub acceleration: Vec2,
    pub last_acceleration: Vec2,
    pub class: Class,
    pub shots_fired: u32,
    pub tick_updated: u32,
}
impl Target {
    pub fn new(position: Vec2, velocity: Vec2, class: Class) -> Target {
        Target {
            position,
            velocity,
            last_velocity: velocity,
            acceleration: Vec2::zero(),
            last_acceleration: Vec2::zero(),
            class,
            shots_fired: 0,
            tick_updated: current_tick(),
        }
    }
    pub fn sanity_check(&self, new_position: Vec2, new_velocity: Vec2) -> bool {
        let dt = (current_tick() - self.tick_updated) as f64 * TICK_LENGTH;
        let acceleration = (new_velocity - self.last_velocity) / dt;
        let max_acceleration = match self.class {
            Class::Fighter => vec2(160.0, 30.).length(),
            Class::Frigate => vec2(10.0, 5.0).length(),
            Class::Missile => vec2(400.0, 100.0).length(),
            Class::Cruiser => vec2(5.0, 2.5).length(),
            Class::Torpedo => vec2(70.0, 20.0).length(),
            _ => 0.0,
        };
        if acceleration.length() > max_acceleration {
            debug!("Acceleration too high. Clamping.");
            return false;
        }
        if (new_position - self.position).length() > new_velocity.length() * dt * 2.0 {
            debug!("Position too far away. Clamping.");
            return false;
        }
        true
    }
    pub fn update(&mut self, new_position: Vec2, new_velocity: Vec2) {
        let dt = (current_tick() - self.tick_updated) as f64 * TICK_LENGTH;
        self.position = new_position;
        self.last_acceleration = self.acceleration;
        self.velocity = new_velocity;
        self.acceleration = (new_velocity - self.last_velocity) / dt;
        self.last_velocity = new_velocity;
        self.tick_updated = current_tick();
    }
    pub fn tick(&mut self) {
        self.velocity += self.acceleration * TICK_LENGTH;
        self.position += self.velocity * TICK_LENGTH;
    }
    pub fn load_radar(&self) {
        set_radar_heading((self.position - position()).angle());
        set_radar_width(angle_at_distance(position().distance(self.position), 50.0));
        set_radar_max_distance((self.position - position()).length() + 20.0);
        set_radar_min_distance((self.position - position()).length() - 20.0);
    }
}
#[derive(Debug, PartialEq)]
pub struct TentativeTarget {
    pub positions: Vec<Vec2>,
    pub average_position: Vec2,
    pub class: Class,
}
