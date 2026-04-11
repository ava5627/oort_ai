use oort_api::prelude::*;

use crate::kalman_filter::KalmanFilter;
use crate::target::Target;

pub struct KalmanTest {
    kalman_filter: KalmanFilter,
    target: Option<Target>,
    real_target: Option<Target>,
}
impl Default for KalmanTest {
    fn default() -> Self {
        Self::new()
    }
}

impl KalmanTest {
    pub fn new() -> KalmanTest {
        let enemy_x = 19000.0;
        let enemy_y = -19000.0;
        debug!("spawn fighter team 0 position (100, 0) heading 0");
        debug!(
            "spawn fighter team 1 position ({}, {}) heading 0",
            enemy_x, enemy_y
        );
        KalmanTest {
            kalman_filter: KalmanFilter::new(),
            target: None,
            real_target: None,
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
        // self.kalman_filter.run();
        self.kalman_filter.once();
        self.kalman_filter.draw();
        self.kalman_filter.point_radar();

        // set_radar_heading(vec2(19e3-100.0, -19e3).angle());
        // set_radar_width(TAU / 300.0);
        // set_radar_max_distance(vec2(19e3 - 100.0, -19e3).length() + 100.0);
        // set_radar_min_distance(vec2(19e3 - 100.0, -19e3).length() - 100.0);
        let predicted_position = self.kalman_filter.predicted_position;
        let predicted_velocity = self.kalman_filter.predicted_velocity;
        if let Some(target) = &mut self.target {
            target.update(predicted_position, contact_velocity);
            target.order = 2;
        } else {
            self.target = Some(Target::new(
                predicted_position,
                predicted_velocity,
                Class::Fighter,
            ));
        }
        let target = self.target.as_mut().unwrap();
        // target.draw_path();
        debug!("target: {}", target);

        let (real_p, real_v) = if let Some(msg) = receive() {
            (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
        } else {
            return;
        };
        let real_target = if let Some(real_target) = &mut self.real_target {
            real_target.update(real_p, real_v);
            real_target.order = 3;
            real_target
        } else {
            self.real_target = Some(Target::new(real_p, real_v, Class::Fighter));
            self.real_target.as_mut().unwrap()
        };
        debug!("real target: {}", real_target);
        real_target.color = 0xff0000;
        real_target.draw_path();
    }
}
