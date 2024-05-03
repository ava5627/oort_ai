// Tutorial: Acceleration
// Fly through the target circle.
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
        // Hint: uncomment me
        debug!("{}", current_tick());
        activate_ability(Ability::Boost);
        let acceleration = if current_tick() < 119 {
            vec2(160.0, 30.0)
        } else {
            vec2(60.0, 30.0)
        };
        let angle = -(acceleration).angle();

        debug!(
            "{} {} ",
            angle,
            active_abilities().get_ability(Ability::Boost)
        );
        debug!("pos {}", position());
        debug!("vel {} {}", velocity().length(), velocity());
        debug!(
            "acc {} {}",
            acceleration.length(),
            acceleration.rotate(heading())
        );
        debug!("a2 {}", acceleration);
        accelerate(vec2(60.0, 30.0).rotate(angle));
        let error = angle_diff(heading(), angle);
        if error.abs() > angle.abs() / 2.0 {
            debug!("tq 1");
            torque(-max_angular_acceleration());
        } else if error.abs() > 0.02 {
            debug!("tq 2");
            torque(max_angular_acceleration());
        } else {
            turn(10.0 * error);
        }
        debug!("{}", error);
    }
}
