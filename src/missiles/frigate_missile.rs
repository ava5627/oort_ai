use crate::missiles::Missile;
use crate::target::Target;
use crate::utils::angle_at_distance;
use crate::utils::boost;
use crate::utils::boost_max_acceleration;
use crate::utils::seek;
use crate::utils::VecUtils;
use crate::utils::{max_accelerate, turn_to};
use oort_api::prelude::*;

pub struct FrigateMissile {
    target: Option<Target>,
    boost_time: Option<usize>,
}

impl Missile for FrigateMissile {
    fn new() -> FrigateMissile {
        set_radar_heading(PI);
        FrigateMissile {
            target: None,
            boost_time: None,
        }
    }
    fn tick(&mut self) {
        let (target_position, target_velocity) =
            if let Some(contact) = scan().filter(|c| c.class != Class::Missile && self.target.is_some()) {
                (contact.position, contact.velocity)
            } else if let Some(msg) = receive() {
                (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
            } else if let Some(contact) = scan().filter(|c| c.class != Class::Missile) {
                (contact.position, contact.velocity)
            } else {
                set_radar_heading(radar_heading() + radar_width());
                set_radar_width(TAU / 4.0);
                set_radar_max_distance(1e99);
                set_radar_min_distance(0.0);
                accelerate(vec2(100.0, 0.0).rotate(heading()));
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
        let behind = self.target_behind_frigate(target_position);
        if behind {
            let diff = angle_diff(target_position.angle(), position().angle());
            if diff > 0.0 {
                turn_to(position().rotate(-PI / 2.0).angle());
            } else {
                turn_to(position().rotate(PI / 2.0).angle());
            }
            accelerate(vec2(100.0, 0.0).rotate(heading()));
        } else {
            self.seek_target();
        }
    }
}

impl FrigateMissile {
    fn seek_target(&mut self) {
        let target = self.target.as_ref().unwrap();
        let dp = target.position - position();
        if dp.length() > 500.0 {
            seek(target);
        } else {
            let ma = boost_max_acceleration();
            max_accelerate(vec2(ma.x, -ma.y).rotate(dp.angle()));
            turn_to(dp.angle());
        }
        if dp.length() < 195.0 {
            explode();
        }
        let error = angle_diff(dp.angle(), heading()).abs();
        let should_boost = error < PI / 4.0;
        boost(should_boost, &mut self.boost_time);
    }

    fn target_behind_frigate(&self, target_position: Vec2) -> bool {
        let target_angle = target_position.angle();
        let missile_angle = position().angle();
        let diff = angle_diff(target_angle, missile_angle);
        PI - diff.abs() < PI / 13.0
    }
}
