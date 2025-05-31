use oort_api::prelude::*;

use crate::kalman_filter::KalmanFilter;
use crate::target::Target;
use crate::utils::{turn_to, VecUtils};

pub struct Test {
    kalman_filter: KalmanFilter,
    target: Option<Target>,
}
impl Default for Test {
    fn default() -> Self {
        Self::new()
    }
}

impl Test {
    pub fn new() -> Test {
        let enemy_x = 19000.0;
        let enemy_y = -19000.0;
        debug!("spawn fighter team 0 position (100, 0) heading 0");
        debug!(
            "spawn fighter team 1 position ({}, {}) heading 0",
            enemy_x, enemy_y
        );
        Test {
            kalman_filter: KalmanFilter::new(),
            target: None,
        }
    }
    pub fn tick(&mut self) {
        let (contact_position, contact_velocity, snr) = if let Some(contact) = scan() {
            (contact.position, contact.velocity, contact.snr)
        } else {
            set_radar_heading(radar_heading() - radar_width());
            set_radar_width(TAU / 40.0);
            set_radar_max_distance(1e100);
            set_radar_min_distance(0.0);
            return;
        };

        self.kalman_filter
            .add_measurement(contact_position, contact_velocity, snr);
        self.kalman_filter.run();
        self.kalman_filter.point_radar();
        let predicted_position = self.kalman_filter.predicted_position;
        if let Some(target) = &mut self.target {
            target.update(predicted_position, contact_velocity);
        } else {
            self.target = Some(Target::new(
                predicted_position,
                contact_velocity,
                Class::Fighter,
            ));
        }
        let target = self.target.as_mut().unwrap();
        let prediciton = if position().distance(target.position) < 3000.0 {
            target.lead(0)
        } else {
            target.position
        };
        let angle = position().angle_to(prediciton);
        turn_to(angle);
        accelerate(Vec2::angle_length(angle, max_forward_acceleration()));
        let miss_by = angle_diff(heading(), angle).abs() * prediciton.length();
        if prediciton.length() < 3000.0 && miss_by < 20.0 {
            fire(0);
        }
        if angle_diff(heading(), angle).abs() < PI / 10.0 {
            activate_ability(Ability::Boost);
        }
    }
}
