use oort_api::prelude::*;

use crate::target::Target;
use crate::utils::turn_to;
pub struct Ship {
    target: Target,
}
impl Ship {
    pub fn new() -> Ship {
        Ship {
            target: Target::new(target(), target_velocity(), Class::Fighter),
        }
    }
    pub fn tick(&mut self) {
        self.target.update(target(), target_velocity());
        let predicted_position = self.target.lead(0);
        self.target.draw_path();
        let angle = predicted_position.angle();
        turn_to(angle);
        if angle_diff(heading(), angle).abs() < PI / 10.0 {
            activate_ability(Ability::Boost);
        }
        let miss_by = angle_diff(heading(), angle) * predicted_position.length();
        if miss_by.abs() < 10.0 {
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
