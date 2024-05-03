// Tutorial: Acceleration 2
// Fly through the target circle. The target is in a random
// location given by the "target" function.
//
// You can add vectors together: vec2(a, b) + vec2(c, d) == vec2(a + c, b + d)
// And subtract them: vec2(a, b) - vec2(c, d) == vec2(a - c, b - d)
use oort_api::prelude::*;

pub struct Ship {
    mode: usize,
}

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship { mode: 0 }
    }

    pub fn tick(&mut self) {
        // Hint: "target() - position()" returns a vector pointing towards the target.
        self.turn_to((target() - position()).angle());
        accelerate(target() - position());
    }

    fn turn_to(&mut self, target_heading: f64) {
        let heading_error = angle_diff(heading(), target_heading);
        debug!("{}", heading_error);
        debug!("{}", angle_diff(0.0, target_heading));
        if self.mode == 3 {
            activate_ability(Ability::Boost);
        }
        if heading_error.abs() > angle_diff(0.0, target_heading).abs() / 2.0 {
            self.mode = 1;
            debug!("1 {}", max_angular_acceleration() * heading_error.signum());
            torque(max_angular_acceleration() * heading_error.signum())
        } else if heading_error.abs() > 0.02 && self.mode == 1 {
            self.mode = 2;
            debug!("2 {}", max_angular_acceleration() * -heading_error.signum());
            torque(max_angular_acceleration() * -heading_error.signum())
        } else {
            self.mode = 3;
            turn(10. * heading_error);
        }
    }
}
