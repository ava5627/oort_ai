use crate::pid::PID;
use oort_api::prelude::*;
const BULLET_SPEED: f64 = 1000.0; // m/s
pub struct Ship {
    has_contact: bool,
    target_last_velocity: Vec2,
    pid: PID,
}
impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            pid: PID::new(50.0, 0.0, 1000.0, 1.0, max_angular_acceleration()),
            has_contact: false,
            target_last_velocity: vec2(0.0, 0.0),
        }
    }
    pub fn tick(&mut self) {
        if !self.has_contact {
            set_radar_heading(radar_heading() + radar_width());
        }
        if let Some(contact) = scan() {
            self.has_contact = true;
            let contact_heading = (contact.position - position()).angle();
            set_radar_heading(contact_heading);
            set_radar_width(1.0 / ((contact.position - position()).length() / 100.0));
            let future_pos = self.predict_position(contact.position, contact.velocity);
            draw_triangle(future_pos, 10.0, 0xffffff);
            draw_square(contact.position, 10.0, 0xff0000);
            draw_line(
                contact.position,
                contact.position + contact.velocity,
                0xff0000,
            );
            draw_line(
                position(),
                vec2(5000.0, 0.0).rotate(heading()) + position(),
                0x00ff00,
            );
            let target_heading = (future_pos - position()).angle();
            let error = angle_distance(heading(), target_heading);
            let av = self.pid.update(error);
            activate_ability(Ability::Boost);
            accelerate(vec2(1000.0, 0.0).rotate(heading()));
            torque(av);
            if (future_pos - position()).length() < 6000.0 {
                fire(0);
            }
            debug!("contact heading: {}", contact_heading);
            debug!(
                "distance to contact: {}",
                (contact.position - position()).length()
            );
            debug!("distance to future: {}", (future_pos - position()).length());
        } else {
            self.has_contact = false;
            self.target_last_velocity = vec2(0.0, 0.0);
            set_radar_width(1.0);
            self.pid.reset();
        }
    }
    pub fn predict_position(&mut self, target_position: Vec2, target_velocity: Vec2) -> Vec2 {
        let relative_velocity = target_velocity - velocity();
        let relative_position = target_position - position();
        let acceleration = relative_velocity - self.target_last_velocity;
        let time_to_target =
            relative_position.length() / (BULLET_SPEED + relative_velocity.length());
        let new_position = relative_position
            + relative_velocity * time_to_target
            + acceleration * time_to_target.powf(2.0) / 2.0;
        self.target_last_velocity = relative_velocity;
        new_position + position()
    }
}
pub fn angle_distance(a: f64, b: f64) -> f64 {
    (a - b + 3.0 * PI) % (2.0 * PI) - PI
}
