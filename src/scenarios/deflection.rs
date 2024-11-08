use std::collections::VecDeque;

use oort_api::prelude::*;

use crate::target::Target;
use crate::utils::{draw_curve, lead_target, turn_to, VecUtils};
pub struct Ship {
    target: Target,
    predictions: VecDeque<Vec2>,
    real_positions: VecDeque<Vec2>,
}
impl Ship {
    pub fn new() -> Ship {
        Ship {
            target: Target::new(target(), target_velocity(), Class::Fighter),
            predictions: VecDeque::new(),
            real_positions: VecDeque::new(),
        }
    }
    pub fn tick(&mut self) {
        self.target.update(target(), target_velocity());
        let predicted_position = lead_target(&self.target, 0);
        self.predictions.push_back(predicted_position + position());
        if self.predictions.len() > 300 {
            self.predictions.pop_front();
        }
        self.real_positions.push_back(target());
        if self.real_positions.len() > 300 {
            self.real_positions.pop_front();
        }
        draw_curve(&self.predictions, 0x00ff00, false);
        draw_curve(&self.real_positions, 0xff0000, false);
        draw_triangle(predicted_position + position(), 10.0, 0xff0000);
        draw_line(position(), predicted_position + position(), 0xff0000);
        draw_line(
            position(),
            position() + Vec2::angle_length(heading(), predicted_position.length()),
            0x00ff00,
        );
        let angle = predicted_position.angle();
        turn_to(angle);
        if angle_diff(heading(), angle).abs() < PI / 10.0 {
            activate_ability(Ability::Boost);
        }
        if angle_diff(heading(), angle).abs() < PI / 15.0 {
            fire(0);
        }
        accelerate(predicted_position);
    }
}

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}
