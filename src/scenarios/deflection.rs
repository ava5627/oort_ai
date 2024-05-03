use oort_api::prelude::*;

pub struct PID {
    pub p: f64,
    pub i: f64,
    pub d: f64,
    pub last_error: Option<f64>,
    pub integral: f64,
    pub integral_limit: f64,
    pub output_limit: f64,
    pub delta_time: f64,
}

impl PID {
    pub fn new(p: f64, i: f64, d: f64, integral_limit: f64, output_limit: f64) -> PID {
        PID {
            p,
            i,
            d,
            last_error: None,
            integral: 0.0,
            integral_limit,
            output_limit,
            delta_time: TICK_LENGTH,
        }
    }

    pub fn update(&mut self, error: f64) -> f64 {
        let p = self.p * error;
        self.integral += error * self.delta_time;
        self.integral = self
            .integral
            .clamp(-self.integral_limit, self.integral_limit);
        let i = self.i * self.integral;
        let derivative = match self.last_error {
            Some(last_error) => angle_diff(error, last_error) / self.delta_time,
            None => 0.0,
        };
        self.last_error = Some(error);
        let d = self.d * derivative;
        let output = p + i + d;
        output.clamp(-self.output_limit, self.output_limit)
    }

    pub fn reset(&mut self) {
        self.last_error = None;
        self.integral = 0.0;
    }
}

pub struct Ship {
    last_velocity: Option<Vec2>,
    last_angle: Option<f64>,
    last_dangle: Option<f64>,
}

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            last_velocity: None,
            last_angle: None,
            last_dangle: None,
        }
    }

    pub fn tick(&mut self) {
        // Hint: "angle_diff(heading(), (target() - position()).angle())"
        // returns the direction your ship needs to turn to face the target.
        let predicted_position = self.lead_target(target(), target_velocity(), 1000.0);
        let angle = predicted_position.angle();
        self.turn_to_target(angle);
        if angle_diff(heading(), angle).abs() < 0.005 {
            fire(0);
        }
        accelerate(predicted_position);
        draw_line(position(), target(), 0xffffff);
    }

    pub fn turn_to_target(&mut self, target_heading: f64) {
        let error = angle_diff(target_heading, heading());
        debug!("error: {}", error);
        let delta_angle = match self.last_angle {
            Some(last_angle) => angle_diff(last_angle, target_heading) / TICK_LENGTH,
            None => 0.0,
        };
        debug!("delta_angle: {}", delta_angle);
        let target_angular_acceleration = match self.last_dangle {
            Some(last_dangle) => delta_angle - last_dangle,
            None => 0.0,
        };
        let time_to_stop = (angular_velocity() - delta_angle).abs() / max_angular_acceleration();
        debug!("desired_angular_velocity: {}", delta_angle);
        debug!("time_to_stop: {}", time_to_stop);
        let target_when_stopped = target_heading + delta_angle * time_to_stop
            - 0.5 * target_angular_acceleration * time_to_stop.powi(2);
        let heading_when_stopped = heading() + angular_velocity() * time_to_stop
            - 0.5 * max_angular_acceleration() * time_to_stop.powi(2);
        let heading_when_stopped_next_tick = heading_when_stopped
            + angular_velocity() * TICK_LENGTH
            - 0.5 * max_angular_acceleration() * TICK_LENGTH.powi(2);

        let predicted_error = angle_diff(heading_when_stopped_next_tick, target_when_stopped);
        let stopping_torque = (delta_angle - angular_velocity()) / TICK_LENGTH;
        debug!("predicted_error: {}", predicted_error);
        if error.abs() < 0.001 && stopping_torque.abs() < max_angular_acceleration() {
            debug!("tq {}", stopping_torque);
            torque(stopping_torque);
        } else if predicted_error > 0.0 {
            debug!("tq {}", max_angular_acceleration());
            torque(max_angular_acceleration());
        } else {
            debug!("tq {}", -max_angular_acceleration());
            torque(-max_angular_acceleration());
        }

        self.last_angle = Some(target_heading);
        self.last_dangle = Some(delta_angle);
    }

    fn lead_target(
        &mut self,
        target_position: Vec2,
        target_velocity: Vec2,
        bullet_speed: f64,
    ) -> Vec2 {
        let delta_position = target_position - position();
        let delta_velocity = target_velocity - velocity();
        // rotate delta velocity into self.last_velocities and remove the oldest
        let last_velocity = match self.last_velocity {
            Some(last_velocity) => last_velocity,
            None => delta_velocity,
        };
        let acceleration = (delta_velocity - last_velocity) / TICK_LENGTH;
        self.last_velocity = Some(delta_velocity);
        let mut prediction = delta_position;
        for _ in 0..100 {
            let time_to_target = prediction.length() / bullet_speed;
            let new_prediction = delta_position
                + delta_velocity * time_to_target
                + 0.5 * acceleration * time_to_target.powi(2);
            let error = (new_prediction - prediction).length();
            prediction = new_prediction;
            if error < 1e-3 {
                break;
            }
        }
        draw_triangle(prediction + position(), 10.0, 0x00ff00);
        draw_line(position(), prediction + position(), 0x00ff00);
        self.draw_heading(position().distance(prediction + position()));
        debug!("delta_position: {}", delta_position);
        debug!("delta_velocity: {}", delta_velocity);
        debug!("acceleration: {}", acceleration);
        prediction
    }

    fn draw_heading(&self, distance: f64) {
        draw_line(
            position(),
            position() + vec2(distance, 0.0).rotate(heading()),
            0xff0000,
        );
    }
}
