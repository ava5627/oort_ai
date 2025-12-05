use oort_api::prelude::*;

use crate::target::Target;
use crate::utils::angle_at_distance;
use crate::utils::boost;
use crate::utils::seek;
use crate::utils::turn_to;
use crate::utils::VecUtils;

pub enum Ship {
    Missile(Missile),
    Fighter,
}

const SEEDS: [u128; 10] = [
    949848, 6772418, 5702349, 14485900, 7399742, 6136476, 12549780, 2923716, 9414401, 3461066,
];

impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        if class() == Class::Fighter {
            set_radar_heading(PI);
        }
        match class() {
            Class::Missile => Ship::Missile(Missile::new()),
            Class::Fighter => Ship::Fighter,
            _ => unreachable!(),
        }
    }

    pub fn tick(&mut self) {
        if let Ship::Missile(m) = self {
            m.tick();
            return;
        }
        let seed_r = seed();
        let index = SEEDS.iter().position(|&s| s == seed_r).unwrap_or(10);
        if [0, 1, 3, 6, 9].contains(&index) {
            fire(1);
        }
        if let Some(contact) = scan() {
            let angle = (contact.position - position()).angle();
            torque(angle_diff(heading(), angle).signum() * 100.0);
            let gain = match index {
                7 => PI / 6.0,
                _ => PI / 16.0,
            };

            if angle_diff(heading(), angle).abs() < gain {
                fire(1)
            }
            set_radar_heading(angle);
            set_radar_width(angle_at_distance(
                position().distance(contact.position),
                100.0,
            ));
            if angle_diff(angle, heading()).abs() < PI / 3.0 {
                activate_ability(Ability::Boost);
            }
            send([
                contact.position.x,
                contact.position.y,
                contact.velocity.x,
                contact.velocity.y,
            ]);
            accelerate(vec2(400.0, 0.0).rotate(angle));
        } else {
            set_radar_heading(radar_heading() + radar_width());
            set_radar_width(TAU / 4.0);
        }
    }
}

pub struct Missile {
    target: Option<Target>,
    boost_time: Option<usize>,
}

impl Default for Missile {
    fn default() -> Self {
        Self::new()
    }
}

impl Missile {
    pub fn new() -> Missile {
        set_radar_heading(PI / 6.0);
        set_radar_width(TAU / 4.0);
        Missile {
            target: None,
            boost_time: None,
        }
    }
    pub fn tick(&mut self) {
        let (target_position, target_velocity) = if let Some(contact) = scan() {
            (contact.position, contact.velocity)
        } else {
            set_radar_heading(radar_heading() - radar_width());
            set_radar_width(TAU / 4.0);
            if let Some(msg) = receive() {
                (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
            } else {
                accelerate(vec2(400.0, 0.0).rotate(heading()));
                return;
            }
        };
        set_radar_heading((target_position - position()).angle());
        set_radar_width(angle_at_distance(
            position().distance(target_position),
            100.0,
        ));
        if let Some(target) = &mut self.target {
            if target_position.distance(target.position) < 100.0 {
                target.update(target_position, target_velocity);
            }
        } else {
            self.target = Some(Target::new(
                target_position,
                target_velocity,
                Class::Fighter,
            ));
        }
        let target = self.target.as_mut().unwrap();
        let should_boost =
            angle_diff((target.position - position()).angle(), heading()).abs() < PI / 4.0;
        boost(should_boost, &mut self.boost_time);
        self.seek();
    }

    pub fn seek(&mut self) {
        let target = self.target.as_mut().unwrap();
        let dp = target.position - position();
        let dv = target.velocity - velocity();
        if dp.length() > 940.0 {
            seek(target);
        } else {
            let future_pos = dp + dv * (11.0 * TICK_LENGTH);
            missile_accelerate(vec2(300.0, -100.0).rotate(future_pos.angle()));
            turn_to(future_pos.angle() - 0.05);
        }
        let time = 11.0;
        let future_dp = dp
            + target.velocity * (time * TICK_LENGTH)
            + 0.5 * target.acceleration * (time * TICK_LENGTH).powf(2.0);
        let frag_p =
            (velocity() + Vec2::angle_length(heading() + 0.05, 1900.0)) * TICK_LENGTH * time;
        if future_dp.length() - frag_p.length() < 5.0 {
            explode();
        }
    }

    pub fn max_acceleration(&self) -> Vec2 {
        if let Some(t) = self.boost_time {
            if t < 120 {
                vec2(
                    max_forward_acceleration() + 100.0,
                    max_lateral_acceleration(),
                )
            } else {
                vec2(max_forward_acceleration(), max_lateral_acceleration())
            }
        } else {
            vec2(
                max_forward_acceleration() + 100.0,
                max_lateral_acceleration(),
            )
        }
    }
}

pub fn missile_accelerate(a: Vec2) {
    let missile_frame = a.rotate(-heading());
    let x = missile_frame.x.clamp(1e-10, max_forward_acceleration());
    let y = missile_frame
        .y
        .clamp(-max_lateral_acceleration(), max_lateral_acceleration());
    let adjusted = vec2(x, y);
    accelerate(adjusted.rotate(heading()));
}
