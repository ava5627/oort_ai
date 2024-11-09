use crate::utils::{angle_at_distance, bullet_speeds, gun_offsets, VecUtils};
use oort_api::prelude::*;
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub position: Vec2,
    pub velocity: Vec2,
    pub last_velocity: Vec2,
    pub acceleration: Vec2,
    pub last_acceleration: Vec2,
    pub jerk: Vec2,
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
            jerk: Vec2::zero(),
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
        self.velocity = new_velocity;
        self.last_acceleration = self.acceleration;
        self.acceleration = (self.velocity - self.last_velocity) / dt;
        self.jerk = (self.acceleration - self.last_acceleration) / dt;
        self.last_velocity = self.velocity; // set after because velocity is changed in the tick function but we don't know if thats actually accurate
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

    pub fn lead(&self, gun: usize) -> Vec2 {
        let gun_offset = gun_offsets(gun);
        let gun_position = position() - gun_offset.rotate(heading());
        let dp = self.position - gun_position;
        let dv = self.velocity - velocity();

        let bullet_speed = bullet_speeds(gun);

        let mut future_position = dp;
        for _ in 0..100 {
            let time_to_target = future_position.length() / bullet_speed;
            let new_future_position = dp
                + dv * time_to_target
                + self.acceleration * time_to_target.powi(2) / 2.0
                + self.jerk * time_to_target.powi(3) / 6.0;
            if (future_position - new_future_position).length() < 1e-3 {
                break;
            }
            future_position = new_future_position;
        }
        if future_position.x.is_nan() || future_position.y.is_nan() {
            self.position
        } else {
            future_position
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct TentativeTarget {
    pub positions: Vec<Vec2>,
    pub average_position: Vec2,
    pub class: Class,
}
