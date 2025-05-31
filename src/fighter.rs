use crate::pid::PID;
use crate::utils::VecUtils;
use crate::utils::{angle_at_distance, draw_curve, draw_heading, send_class_and_position, turn_to};
use oort_api::prelude::*;
use std::collections::VecDeque;
pub struct Fighter {
    move_to: Vec2,
    last_velocity: Option<Vec2>,
    accelerations: VecDeque<Vec2>,
    predictions: VecDeque<Vec2>,
    real_positions: VecDeque<Vec2>,
    pid: PID,
}
impl Fighter {
    pub fn new() -> Fighter {
        let pid = PID::new(
            50.0,
            0.0,
            1000.0 / 60.0,
            max_angular_acceleration(),
            max_angular_acceleration(),
        );
        Fighter {
            move_to: position(),
            last_velocity: None,
            accelerations: VecDeque::new(),
            pid,
            predictions: VecDeque::new(),
            real_positions: VecDeque::new(),
        }
    }
    pub fn tick(&mut self) {
        select_radio(1);
        set_radio_channel(9);
        send_class_and_position();
        select_radio(0);
        set_radio_channel(0);
        debug!("Hello from fighter.rs");
        fire(1);
        fire(0);
        let (target, target_velocity) = if let Some(contact) = scan() {
            (contact.position, contact.velocity)
        } else {
            fire(0);
            set_radar_heading(radar_heading() + radar_width());
            self.pid.reset();
            self.last_velocity = None;
            self.accelerations.clear();
            set_radar_width(TAU / 30.0);
            set_radar_max_distance(1e9);
            set_radar_min_distance(0.0);
            return;
        };
        set_radar_width(angle_at_distance(position().distance(target), 100.0));
        set_radar_heading(target.angle_to(position()));
        set_radar_max_distance(position().distance(target) + 100.0);
        set_radar_min_distance(position().distance(target) - 100.0);
        send([target.x, target.y, target_velocity.x, target_velocity.y]);
        let predicted_position = self.lead_target(target, target_velocity, 1000.0);
        self.real_positions.push_back(target);
        if self.real_positions.len() > 300 {
            self.real_positions.pop_front();
        }
        draw_curve(&self.predictions, 0xff0000, false);
        draw_curve(&self.real_positions, 0x00ff00, false);
        let angle = predicted_position.angle();
        let random_offset = rand(-1.0, 1.0) * TAU / 240.0;
        turn_to(angle + random_offset);
        fire(0);
        if angle_diff(heading(), target_velocity.angle()).abs() < 0.1
            && angle_diff(heading(), angle).abs() < 0.1
        {
            activate_ability(Ability::Boost);
        } else {
            deactivate_ability(Ability::Boost);
        }
        self.move_to = target;
        let acceleration_vector =
            vec2(max_forward_acceleration(), 0.0).rotate((self.move_to - position()).angle());
        accelerate(acceleration_vector);
    }
    fn lead_target(
        &mut self,
        target_position: Vec2,
        target_velocity: Vec2,
        bullet_speed: f64,
    ) -> Vec2 {
        let delta_position = target_position - position();
        let delta_velocity = target_velocity - velocity();
        let last_velocity = match self.last_velocity {
            Some(last_velocity) => last_velocity,
            None => delta_velocity,
        };
        self.last_velocity = Some(target_velocity);
        let current_acceleration = (target_velocity - last_velocity) / TICK_LENGTH;
        self.accelerations.push_back(current_acceleration);
        if self.accelerations.len() > 10 {
            self.accelerations.pop_front();
        }
        let mut acceleration = vec2(0.0, 0.0);
        for a in self.accelerations.iter() {
            acceleration += a;
        }
        acceleration /= self.accelerations.len() as f64;
        let mut prediction = delta_position;
        let mut time_to_target = 0.0;
        for i in 0..100 {
            time_to_target = prediction.length() / bullet_speed;
            let bullet_velocity = vec2(bullet_speed, 0.0).rotate(prediction.angle());
            let velocity_at_hit = delta_velocity + acceleration * time_to_target;
            if (bullet_velocity - velocity_at_hit).length() < 1e-3 {
                debug!("hit not possible");
                return target_position;
            }
            let new_prediction = delta_position
                + delta_velocity * time_to_target
                + 0.5 * acceleration * time_to_target.powi(2);
            let error = (new_prediction - prediction).length();
            prediction = new_prediction;
            if error < 1e-3 {
                debug!("iterations: {}", i);
                break;
            }
        }
        draw_triangle(prediction + position(), 10.0, 0x00ff00);
        let real_future_position = target_position
            + target_velocity * time_to_target
            + 0.5 * acceleration * time_to_target.powi(2);
        draw_triangle(real_future_position, 10.0, 0x0000ff);
        draw_line(position(), prediction + position(), 0x00ff00);
        draw_heading(position().distance(prediction + position()));
        self.predictions.push_back(real_future_position);
        if self.predictions.len() > 300 {
            self.predictions.pop_front();
        }
        debug!("prediction 0: {}", self.predictions[0]);
        debug!("target_position: {}", target_position);
        prediction
    }
}

impl Default for Fighter {
    fn default() -> Self {
        Self::new()
    }
}
