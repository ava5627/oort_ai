use crate::missiles::Missile;
use crate::target::Target;
use crate::utils::angle_at_distance;
use crate::utils::boost;
use crate::utils::boost_max_acceleration;
use crate::utils::final_approach;
use crate::utils::seek;
use crate::utils::VecUtils;
use crate::utils::{max_accelerate, turn_to, turn_to_simple};
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
            if let Some(contact) = scan().filter(|c| c.class != Class::Missile) {
                (contact.position, contact.velocity)
            } else if let Some(msg) = receive() {
                (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
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
        if let Some(target) = &self.target {
            let dp = target.position - position();
            if dp.length() > 600.0 {
                seek(target);
            } else {
                let ma = boost_max_acceleration();
                max_accelerate(vec2(ma.x, -ma.y).rotate(dp.angle()));
                turn_to(dp.angle());
            }
            if dp.length() < 195.0 {
                explode();
            }
        }
        boost(
            angle_diff((target_position - position()).angle(), heading()).abs() < PI / 4.0,
            &mut self.boost_time,
        );
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
        let ma = boost_max_acceleration();
        let angle = ma.angle();
        let target_angle = a.angle();
        accelerate(a);
        if dp.length() > 500.0 {
            max_accelerate(vec2(300.0, -100.0).rotate(target_angle + angle));
            turn_to_simple(a.angle() + angle);
        } else {
            max_accelerate(vec2(300.0, -100.0).rotate(dp.angle()));
            turn_to(dp.angle());
        }
        if dp.length() < 195.0 {
            explode();
        }
    }
}
