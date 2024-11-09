use crate::missiles::Missile;
use crate::target::Target;
use crate::utils::angle_at_distance;
use crate::utils::boost;
use crate::utils::final_approach;
use crate::utils::seek;
use crate::utils::VecUtils;
use oort_api::prelude::*;
pub struct FighterMissile {
    target: Option<Target>,
    boost_time: Option<usize>,
}

impl Missile for FighterMissile {
    fn new() -> FighterMissile {
        set_radar_heading(PI);
        FighterMissile {
            target: None,
            boost_time: None,
        }
    }
    fn tick(&mut self) {
        let (target_position, target_velocity) = if let Some(contact) =
            scan().filter(|c| ![Class::Missile, Class::Torpedo].contains(&c.class))
        {
            debug!("contact {:?}", contact);
            (contact.position, contact.velocity)
        } else if let Some(msg) = receive() {
            (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
        } else {
            set_radar_heading(radar_heading() + radar_width());
            set_radar_width(TAU / 4.0);
            accelerate(vec2(100.0, 0.0).rotate(heading()));
            return;
        };
        if let Some(target) = &mut self.target {
            target.update(target_position, target_velocity);
        } else {
            self.target = Some(Target::new(
                target_position,
                target_velocity,
                Class::Missile,
            ));
        }
        debug!("target_position {:?}", target_position);
        set_radar_heading(position().angle_to(target_position));
        set_radar_width(angle_at_distance(
            position().distance(target_position),
            100.0,
        ));
        let target = self.target.as_ref().unwrap();
        let dp = target.position - position();
        if dp.length() > 500.0 {
            seek(target);
        } else {
            final_approach(target);
        }
        let error = angle_diff(heading(), dp.angle());
        let should_boost = error.abs() < 2.0;
        boost(should_boost, &mut self.boost_time);
    }
}
