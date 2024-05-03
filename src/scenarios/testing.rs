use crate::{utils::turn_to, vec_utils::VecUtils};
use oort_api::prelude::*;
pub struct Test {
    time_boost: Option<usize>,
}
impl Default for Test {
    fn default() -> Self {
        Self::new()
    }
}

impl Test {
    pub fn new() -> Test {
        debug!("spawn missile team 0 position (-100, 0) heading 0");
        Test { time_boost: None }
    }
    pub fn tick(&mut self) {
        let max_accel = vec2(max_forward_acceleration(), max_lateral_acceleration());
        let max_boost_accel = max_accel + vec2(100.0, 0.0);
        let angle = max_accel.angle();
        let angle_boost = max_boost_accel.angle();
        let target_position = vec2(000., 000.);
        draw_line(position(), target_position, 0x00ff00);
        let target_angle = position().angle_to(target_position);
        if angle_diff(heading(), target_angle + angle).abs() < 0.1 {
            if self.time_boost.is_none() {
                self.time_boost = Some(0);
            }
            activate_ability(Ability::Boost);
        }
        if let Some(time_boost) = self.time_boost {
            self.time_boost = Some(time_boost + 1);
            if time_boost > 120 {
                turn_to(target_angle + angle);
                accelerate(vec2(300.0, -100.0).rotate(target_angle + angle));
            } else {
                turn_to(target_angle + angle_boost);
                accelerate(vec2(300.0, -100.0).rotate(target_angle + angle_boost));
            }
        } else {
            turn_to(target_angle + angle_boost);
            accelerate(vec2(300.0, -100.0).rotate(target_angle + angle));
        }
    }
}
