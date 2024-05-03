use oort_api::prelude::*;
pub struct Ship {}
impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }
    pub fn tick(&mut self) {
        let predicted_position = self.lead_target(target(), target_velocity(), 1000.0);
        let angle = (predicted_position - position()).angle();
        self.turn_to_target(angle);
        if angle_diff(heading(), angle).abs() < 0.01 {
            fire(0);
        }
        accelerate(vec2(100.0, 0.0).rotate(angle));
    }
    pub fn turn_to_target(&mut self, target_heading: f64) {
        let error = angle_diff(target_heading, heading());
        debug!("error: {}", error);
        let time_to_stop = angular_velocity().abs() / max_angular_acceleration();
        let angle_while_stopping = angular_velocity() * time_to_stop
            - 0.5 * max_angular_acceleration() * time_to_stop.powi(2);
        let stopped_error = angle_diff(target_heading, heading() + angle_while_stopping);
        let applied_torque = max_angular_acceleration() * error.signum();
        debug!("time_to_stop: {}", time_to_stop);
        debug!("angle_while_stopping: {}", angle_while_stopping);
        debug!("stopped_error: {}", stopped_error);
        if stopped_error * error.signum() < 0.0 {
            debug!("stopping");
            debug!("applied_torque: {}", applied_torque);
            torque(applied_torque);
        } else {
            debug!("accelerating");
            debug!("applied_torque: {}", -applied_torque);
            torque(-applied_torque);
        }
    }
    fn lead_target(&self, target_position: Vec2, target_velocity: Vec2, bullet_speed: f64) -> Vec2 {
        let delta_position = target_position - position();
        let delta_velocity = target_velocity - velocity();
        let mut prediction = delta_position;
        for i in 0..100 {
            let time_to_target = prediction.length() / bullet_speed;
            let new_prediction = delta_position + delta_velocity * time_to_target;
            let error = (new_prediction - prediction).length();
            prediction = new_prediction;
            if error < 1e-3 {
                break;
            }
            draw_triangle(prediction, 50.0 - 5.0 * i as f64, 0xff0000);
        }
        draw_triangle(prediction, 10.0, 0x00ff00);
        draw_line(position(), prediction, 0x00ff00);
        self.draw_heading(position().distance(prediction));
        prediction + position()
    }
    fn draw_heading(&self, distance: f64) {
        draw_line(
            position(),
            position() + vec2(distance, 0.0).rotate(heading()),
            0xff0000,
        );
    }
}
