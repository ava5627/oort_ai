use crate::target::{Target, TentativeTarget};
use crate::utils::{angle_at_distance, turn_to};
use oort_api::prelude::*;

pub struct Ship {
    target: Option<Target>,
    tentative_target: TentativeTarget,
}
impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            target: None,
            tentative_target: TentativeTarget::new(),
        }
    }
    pub fn tick(&mut self) {
        if let Some(contact) = scan() {
            if contact.snr < 20.0 {
                self.tentative_target.class = contact.class;
                self.tentative_target.update(contact.position);
                self.tentative_target.load_radar();
                turn_to((self.tentative_target.average_position - position()).angle());
                accelerate(self.tentative_target.average_position);
            } else {
                let contact_heading = (contact.position - position()).angle();
                let dp = contact.position - position();
                set_radar_heading(contact_heading);
                set_radar_width(angle_at_distance(dp.length(), 100.0));
                set_radar_max_distance(dp.length() + 100.0);
                set_radar_min_distance(dp.length() - 100.0);
                if self.target.is_none() {
                    self.target = Some(Target::new(
                        contact.position,
                        contact.velocity,
                        contact.class,
                    ));
                } else if let Some(target) = &mut self.target {
                    target.update(contact.position, contact.velocity);
                }
            }
        } else {
            set_radar_width(TAU / 30.0);
            set_radar_heading(radar_heading() + radar_width() / 2.0);
            set_radar_max_distance(1e100);
            set_radar_min_distance(0.0);
        }

        if let Some(target) = &self.target {
            let prediction = target.lead(0);
            let angle = prediction.angle();
            turn_to(angle);
            let miss_by = angle_diff(heading(), angle) * prediction.length();
            debug!("Miss by: {}", miss_by);
            debug!("Distance to target: {}", target.position.distance(position()));
            if miss_by.abs() < 20.0 && target.position.distance(position()) < 10000.0 {
                fire(0);
            }
            if angle_diff(heading(), angle).abs() < PI / 5.0 {
                activate_ability(Ability::Boost);
            }
        }
    }
}
pub fn angle_distance(a: f64, b: f64) -> f64 {
    (a - b + 3.0 * PI) % (2.0 * PI) - PI
}
