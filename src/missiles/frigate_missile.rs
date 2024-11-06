use crate::missiles::Missile;
use crate::target::Target;
use crate::utils::{angle_at_distance, turn_to, turn_to_simple};
use crate::vec_utils::VecUtils;
use oort_api::prelude::*;

fn find_target() -> Option<(Vec2, Vec2)> {
    set_radar_heading(radar_heading() + radar_width());
    set_radar_width(TAU / 4.0);
    if let Some(msg) = receive() {
        Some((vec2(msg[0], msg[1]), vec2(msg[2], msg[3])))
    } else {
        accelerate(vec2(100.0, 0.0).rotate(heading()));
        None
    }
}

pub struct FrigateMissile {
    target: Option<Target>,
}

impl Missile for FrigateMissile {
    fn new() -> FrigateMissile {
        set_radar_heading(PI);
        FrigateMissile { target: None }
    }
    fn tick(&mut self) {
        let (target_position, target_velocity) = if let Some(contact) = scan() {
            if contact.class != Class::Missile {
                (contact.position, contact.velocity)
            } else if let Some(target) = find_target() {
                target
            } else {
                return;
            }
        } else if let Some(target) = find_target() {
            target
        } else {
            return;
        };
        set_radar_heading(position().angle_to(target_position));
        set_radar_width(angle_at_distance(
            position().distance(target_position),
            100.0,
        ));
        if let Some(target) = &mut self.target {
            if target_position.distance(target.position) < 100.0 {
                target.update(target_position, target_velocity);
            } else {
                self.target = Some(Target::new(
                    target_position,
                    target_velocity,
                    Class::Missile,
                ));
            }
        } else {
            self.target = Some(Target::new(
                target_position,
                target_velocity,
                Class::Missile,
            ));
        }
        self.seek();
        if angle_diff((target_position - position()).angle(), heading()).abs() < 2.0 {
            activate_ability(Ability::Boost);
        }
    }
    fn seek(&mut self) {
        let target = self.target.as_ref().unwrap();
        let dp = target.position - position();
        let dv = target.velocity - velocity();
        let closing_speed = -(dp.y * dv.y - dp.x * dv.x).abs() / dp.length();
        let los = dp.angle();
        let los_rate = dv.wedge(dp) / dp.square_magnitude();
        const N: f64 = 4.0;
        let accel = N * closing_speed * los_rate; // + N * nt.length() / 2.0 * los_rate;
        let a = vec2(100.0, accel).rotate(los);
        let a = Vec2::angle_length(a.angle(), 400.0);
        accelerate(a);
        if dp.length() > 500.0 {
            turn_to_simple(a.angle());
        } else {
            turn_to_simple(dp.angle());
        }
        if dp.length() < 195.0 {
            explode();
        }
    }
}
