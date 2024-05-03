use crate::utils::angle_at_distance;
use crate::utils::turn_to_simple;
use crate::vec_utils::VecUtils;
use crate::missiles::Missile;
use oort_api::prelude::*;
pub struct FighterMissile {
    target_position: Vec2,
    target_velocity: Vec2,
    target_acceleration: Vec2,
}
impl Missile for FighterMissile {
    fn new() -> FighterMissile {
        set_radar_heading(PI);
        FighterMissile {
            target_position: Vec2::zero(),
            target_velocity: Vec2::zero(),
            target_acceleration: Vec2::zero(),
        }
    }
    fn tick(&mut self) {
        let (target_position, target_velocity) = if let Some(contact) = scan() {
            if contact.class == Class::Missile {
                set_radar_heading(radar_heading() + radar_width());
                set_radar_width(TAU / 4.0);
                if let Some(msg) = receive() {
                    (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
                } else {
                    accelerate(vec2(100.0, 0.0).rotate(heading()));
                    return;
                }
            } else {
                (contact.position, contact.velocity)
            }
        } else {
            set_radar_heading(radar_heading() + radar_width());
            set_radar_width(TAU / 4.0);
            if let Some(msg) = receive() {
                (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
            } else {
                accelerate(vec2(100.0, 0.0).rotate(heading()));
                return;
            }
        };
        set_radar_heading(target_position.angle_to(position()));
        set_radar_width(angle_at_distance(
            position().distance(target_position),
            100.0,
        ));
        self.target_acceleration = (target_velocity - self.target_velocity) / TICK_LENGTH;
        self.target_velocity = target_velocity;
        self.target_position = target_position;
        self.seek();
        if angle_diff((self.target_position - position()).angle(), heading()).abs() < 2.0 {
            activate_ability(Ability::Boost);
        }
    }
    fn seek(&mut self) {
        let dp = self.target_position - position();
        let dv = self.target_velocity - velocity();
        let closing_speed = -(dp.y * dv.y - dp.x * dv.x).abs() / dp.length();
        let los = dp.angle();
        let los_rate = dv.wedge(dp) / dp.square_magnitude();
        const N: f64 = 4.0;
        let _nt = self.target_acceleration
            - (self.target_acceleration.dot(dp) / dp.length().powf(2.0)) * dp;
        let accel = N * closing_speed * los_rate; // + N * nt.length() / 2.0 * los_rate;
        let a = vec2(100.0, accel).rotate(los);
        let a = Vec2::angle_length(a.angle(), 400.0);
        accelerate(a);
        turn_to_simple(a.angle());
        if dp.length() < 500.0 {
            turn_to_simple(dp.angle());
        }
        if dp.length() < 180.0 {
            explode();
        }
    }
}
