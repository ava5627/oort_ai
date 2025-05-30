use crate::utils::angle_at_distance;
use crate::utils::bullet_speeds;
use crate::utils::class_max_acceleration;
use crate::utils::gun_color;
use crate::utils::gun_offsets;
use crate::utils::VecUtils;
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

    pub fn sanity_check(&self, new_position: Vec2, new_velocity: Vec2, new_class: Class) -> bool {
        if self.class != new_class {
            return false;
        }
        let dt = (current_tick() - self.tick_updated) as f64 * TICK_LENGTH;
        let acceleration = (new_velocity - self.last_velocity) / dt;
        let max_acceleration = class_max_acceleration(new_class);
        if acceleration.length() > max_acceleration {
            return false;
        }
        if (new_position - self.position).length() > new_velocity.length() * dt * 4.0 {
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
        let ma = class_max_acceleration(self.class);
        self.jerk.x = self.jerk.x.clamp(-ma, ma);
        self.jerk.y = self.jerk.y.clamp(-ma, ma);
        self.last_velocity = self.velocity; // set after because velocity is changed in the tick function but we don't know if thats actually accurate
        self.tick_updated = current_tick();
    }

    pub fn tick(&mut self, i: usize) {
        self.velocity += self.acceleration * TICK_LENGTH;
        self.position += self.velocity * TICK_LENGTH;
        draw_polygon(self.position, 50.0, 8, 0.0, 0xffffff);
        draw_square(self.position, 10.0, 0xffffff);
        draw_text!(self.position, 0xffffff, "{}", i);
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
        if !future_position.x.is_normal() || !future_position.y.is_normal() {
            debug!("Impossible to hit target");
            self.position
        } else {
            let adjusted_position = future_position + gun_position;
            draw_square(adjusted_position, 10.0, gun_color(gun));
            draw_line(gun_position, adjusted_position, gun_color(gun));
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

impl Default for TentativeTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl TentativeTarget {
    pub fn new() -> TentativeTarget {
        TentativeTarget {
            positions: Vec::new(),
            average_position: Vec2::zero(),
            class: Class::Unknown,
        }
    }

    pub fn update(&mut self, position: Vec2) {
        self.positions.push(position);
        if self.positions.len() > 10 {
            self.positions.remove(0);
        }
        self.average_position = self.positions.iter().fold(Vec2::zero(), |acc, &p| acc + p)
            / self.positions.len() as f64;
        if self.positions.len() >= 10 {
            self.positions.remove(0);
        }
    }

    pub fn load_radar(&self) {
        set_radar_heading((self.average_position - position()).angle());
        let dist = 100.0 * (11 - self.positions.len()) as f64;
        debug!("radar width: {}", dist);
        set_radar_width(angle_at_distance(
            position().distance(self.average_position),
            100.0 * (11 - self.positions.len()) as f64,
        ));
        set_radar_max_distance((self.average_position - position()).length() + dist);
        set_radar_min_distance((self.average_position - position()).length() - dist);
    }
}
