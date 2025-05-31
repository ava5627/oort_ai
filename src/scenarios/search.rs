use crate::kalman_filter::KalmanFilter;
use crate::target::Target;
use crate::utils::{turn_to, VecUtils};
use oort_api::prelude::*;

pub struct Ship {
    target: Option<Target>,
    kalman_filter: KalmanFilter,
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
            kalman_filter: KalmanFilter::new(),
        }
    }
    pub fn tick(&mut self) {
        if let Some(contact) = scan() {
            self.kalman_filter
                .add_measurement(contact.position, contact.velocity, contact.snr);
            self.kalman_filter.run();
            self.kalman_filter.point_radar();
            if self.target.is_none() {
                self.target = Some(Target::new(
                    self.kalman_filter.predicted_position,
                    contact.velocity,
                    contact.class,
                ));
            } else if let Some(target) = &mut self.target {
                target.update(self.kalman_filter.predicted_position, contact.velocity);
                target.draw_path();
            }
        } else {
            self.kalman_filter.reset();
            set_radar_width(TAU / 30.0);
            set_radar_heading(radar_heading() + radar_width() / 2.0);
            set_radar_max_distance(1e100);
            set_radar_min_distance(0.0);
        }

        if let Some(target) = &mut self.target {
            let prediction = target.lead(0);
            let angle = prediction.angle();
            turn_to(angle);
            let miss_by = angle_diff(heading(), angle) * prediction.length();
            debug!("Target velocity: {}", target.velocity);
            debug!("Target acceleration: {}", target.acceleration);
            debug!("Target jerk: {}", target.jerk);
            debug!("Miss by: {}", miss_by);
            debug!(
                "Distance to target: {}",
                target.position.distance(position())
            );
            accelerate(Vec2::angle_length(angle, max_forward_acceleration()));
            if miss_by.abs() < 10.0 && target.position.distance(position()) < 10000.0 {
                fire(0);
            }
            if angle_diff(heading(), angle).abs() < PI / 5.0 {
                activate_ability(Ability::Boost);
            }
        }
    }
}
