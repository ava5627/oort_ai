// Tutorial: Radar (solution)
// Destroy the enemy ships. Use your radar to find them.
// Hint: Press 'g' in-game to show where your radar is looking.
// Hint: Press 'n' to single-step.
// Hint: Use the set_radar_heading() function to keep your radar pointed at a
// target, or to search for a new one.
//
// Join the Discord at https://discord.gg/vYyu9EhkKH for Oort discussion and
// tournament results.
use oort_api::prelude::*;

use crate::target::Target;
use crate::utils::turn_to;
use crate::utils::VecUtils;
pub struct Ship {
    target: Option<Target>,
}

impl Ship {
    pub fn new() -> Ship {
        Ship { target: None }
    }

    pub fn tick(&mut self) {
        debug!("Hello from radar.rs");
        if let Some(contact) = scan() {
            if let Some(target) = &mut self.target {
                target.update(contact.position, contact.velocity);
                target.load_radar();
            } else {
                self.target = Some(Target::new(
                    contact.position,
                    contact.velocity,
                    contact.class,
                ));
            }
        } else {
            set_radar_heading(radar_heading() + radar_width());
            set_radar_max_distance(1e100);
            set_radar_min_distance(0.0);
        }
        if let Some(target) = &self.target {
            let prediction = target.lead(0);
            let angle = prediction.angle();
            turn_to(angle);
            let miss_by = angle_diff(angle, heading()) * prediction.length();
            if miss_by.abs() < 19.0 {
                fire(0);
            }
            accelerate(Vec2::angle_length(angle, max_forward_acceleration()));
        }
    }
}

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}
