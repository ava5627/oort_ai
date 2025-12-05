use oort_api::prelude::*;

use crate::utils::turn_to;
use crate::utils::angle_at_distance;
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
    target_position: Vec2,
    last_distance: Vec2,
    target_velocity: Vec2,
    target_acceleration: Vec2,
    boost_time: Option<usize>,
}

impl Default for Missile {
    fn default() -> Self {
        Self::new()
    }
}

impl Missile {
    pub fn new() -> Missile {
        set_radar_heading(PI/6.0);
        set_radar_width(TAU / 4.0);
        Missile {
            target_position: vec2(0.0, 0.0),
            last_distance: vec2(0.0, 0.0),
            target_velocity: vec2(0.0, 0.0),
            target_acceleration: vec2(0.0, 0.0),
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
        self.target_acceleration = (target_velocity - self.target_velocity) / TICK_LENGTH;
        self.target_velocity = target_velocity;
        self.target_position = target_position;
        self.seek();
        if angle_diff((self.target_position - position()).angle(), heading()).abs() < PI / 4.0 {
            activate_ability(Ability::Boost);
            if self.boost_time.is_none() {
                self.boost_time = Some(0);
            }
        }
        if let Some(t) = self.boost_time {
            self.boost_time = Some(t + 1);
        }
    }

    pub fn seek(&mut self) {
        let dp = self.target_position - position();
        let dv = self.target_velocity - velocity();
        let closing_speed = -(dp.y * dv.y - dp.x * dv.x).abs() / dp.length();
        let los = dp.angle();
        let los_rate = (dp.y * dv.x - dp.x * dv.y) / dp.length().powf(2.0);

        const N: f64 = 4.0;
        // let nt = self.target_acceleration
        //     - (self.target_acceleration.dot(dp) / dp.length().powf(2.0)) * dp;
        let accel = N * closing_speed * los_rate; // + N * nt.length() / 2.0 * los_rate;
        let a = vec2(100.0, accel).rotate(los);
        let a = vec2(400.0, 0.0).rotate(a.angle());
        let ma = self.max_acceleration();
        let angle = ma.angle();
        let target_angle = a.angle();
        if dp.length() > 940.0 {
            missile_accelerate(vec2(300.0, -100.0).rotate(target_angle + angle));
            turn_to(a.angle() + angle);
        } else {
            let future_pos = dp + dv * (11.0 * TICK_LENGTH);
            missile_accelerate(vec2(300.0, -100.0).rotate(future_pos.angle()));
            turn_to(future_pos.angle() - 0.05);
        }
        let time = 11.0;
        let future_dp = dp
            + self.target_velocity * (time * TICK_LENGTH)
            + 0.5 * self.target_acceleration * (time * TICK_LENGTH).powf(2.0);
        let frag_p =
            (velocity() + Vec2::angle_length(heading() + 0.05, 1900.0)) * TICK_LENGTH * time;
        if future_dp.length() - frag_p.length() < 5.0 {
            explode();
        }
        self.last_distance = dp;
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
