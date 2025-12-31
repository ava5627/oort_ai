use oort_api::prelude::*;

use crate::utils::{turn_to, VecUtils};

pub struct Test {
    target_heading: f64,
    turning: bool,
}
impl Default for Test {
    fn default() -> Self {
        Self::new()
    }
}

impl Test {
    pub fn new() -> Test {
        debug!("spawn frigate team 0 position (100, 0) heading 0");
        Test {
            target_heading: -5.5 * PI / 30.0,
            turning: true,
        }
    }
    pub fn tick(&mut self) {
        draw_line(
            position(),
            position() + Vec2::angle_length(self.target_heading, 1000.0),
            0x00ff00,
        );
        draw_line(
            position(),
            position() + Vec2::angle_length(heading(), 1000.0),
            0xff0000,
        );
        if self.turning {
            self.turn_to(self.target_heading);
        } else {
            turn_to(self.target_heading);
        }
        let curr_error = angle_diff(self.target_heading, heading());
        if curr_error.abs() < 0.01 && reload_ticks(0) == 0 {
            fire(0);
            self.target_heading = rand(0.0, 2.0*PI);
            self.turning = false;
        }
    }

    fn turn_to(&mut self, target_heading: f64) {
        let av = angular_velocity() * TICK_LENGTH;
        let curr_error = angle_diff(target_heading, heading());
        let aa = max_angular_acceleration() * TICK_LENGTH * TICK_LENGTH;
        debug!("angular velocity: {}", av);

        // let passed = (((8.0 * target_heading / aa + 1.0).sqrt() - 1.0) / 2.0).ceil();
        let passed1 = ((-(aa / 2.0 + av)
            + ((aa / 2.0 + av).powi(2) + 2.0 * aa * curr_error.abs()).sqrt())
            / aa)
            .ceil()
            .abs();
        let passed2 = ((-(aa / 2.0 + av)
            - ((aa / 2.0 + av).powi(2) + 2.0 * aa * curr_error.abs()).sqrt())
            / aa)
            .ceil()
            .abs();
        let (passed, accel_sign) = if passed1 < passed2 {
            (passed1, curr_error.signum())
        } else {
            (passed2, -curr_error.signum())
        };
        let heading_when_stopped =
            heading() + av * passed + aa * accel_sign * (passed.powi(2) + passed) / 2.0;
        let error = angle_diff(target_heading, heading_when_stopped).abs();
        let error_per_tick = error * 2.0 / (passed.powi(2) + passed);
        let accel =
            (max_angular_acceleration() - error_per_tick / TICK_LENGTH / TICK_LENGTH) * accel_sign;
        torque(accel);
        debug!("target_heading: {}", target_heading);
        debug!("ticks to target heading: {}", passed);
        debug!("heading when stopped: {}", heading_when_stopped);
        debug!("curr_error: {}", curr_error);
        debug!("error: {}", error);
        debug!("error_per_tick: {}", error_per_tick);
        debug!("ept: {}", error_per_tick/ TICK_LENGTH / TICK_LENGTH);
        draw_line(
            position(),
            position() + Vec2::angle_length(heading_when_stopped, 1000.0),
            0x00ffff,
        );
        debug!("heading: {}", heading());
        debug!("max angular acceleration: {}", max_angular_acceleration() * accel_sign);
        debug!("applied angular acceleration: {}", accel);
        let aa = accel * TICK_LENGTH * TICK_LENGTH * accel_sign;
        let heading_when_stopped_real =
            heading() + av * passed + aa * (passed.powi(2) + passed) / 2.0 * accel_sign;
        debug!("heading_when_stopped_real: {}", heading_when_stopped_real);
        draw_line(
            position(),
            position() + Vec2::angle_length(heading_when_stopped_real, 1000.0),
            0xffff00,
        );
    }
}
