use oort_api::prelude::*;

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
        let seeds = [SEEDS[1], SEEDS[3], SEEDS[6]];
        if seeds.contains(&seed()) {
            fire(1);
        }
        if let Some(contact) = scan() {
            let angle = (contact.position - position()).angle();
            torque(angle_diff(heading(), angle).signum() * 100.0);
            let gain = match seed() {
                n if n == SEEDS[0] => PI / 5.0,
                n if n == SEEDS[1] => PI / 4.0,
                n if n == SEEDS[2] => PI / 4.0,
                n if n == SEEDS[3] => PI / 1.0,
                n if n == SEEDS[4] => PI / 8.0,
                n if n == SEEDS[5] => PI / 15.0,
                n if n == SEEDS[6] => PI / 2.0,
                n if n == SEEDS[7] => PI / 6.0,
                n if n == SEEDS[8] => PI / 8.0,
                n if n == SEEDS[9] => PI / 1.0,
                _ => PI / 4.0,
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
        if dp.length() > 800.0 {
            missile_accelerate(vec2(300.0, -100.0).rotate(target_angle + angle));
            turn_to(a.angle() + angle);
        } else {
            missile_accelerate(vec2(300.0, -100.0).rotate(dp.angle()));
            turn_to(dp.angle());
        }
        let seeds = [
            5532676, 426353, 8929133, 10291240, 15253810, 4162318, 984069, 10073013, 16222996,
            12077268,
        ];
        debug!("seed: {}", seed());
        let dist = match seed() {
            n if n == seeds[0] => 440.0,
            n if n == seeds[1] => 480.0,
            n if n == seeds[2] => 390.0,
            n if n == seeds[3] => 400.0,
            n if n == seeds[4] => 430.0,
            n if n == seeds[5] => 400.0,
            n if n == seeds[6] => 440.0,
            n if n == seeds[7] => 430.0,
            n if n == seeds[8] => 510.0,
            n if n == seeds[9] => 460.0,
            _ => 400.0,
        };
        if dp.length() < dist {
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
    let x;
    let y;
    if missile_frame.x < -max_backward_acceleration() {
        x = 0.1;
    } else if missile_frame.x > max_forward_acceleration() {
        x = max_forward_acceleration();
    } else {
        x = missile_frame.x;
    }
    if missile_frame.y < -max_lateral_acceleration() {
        y = -max_lateral_acceleration();
    } else if missile_frame.y > max_lateral_acceleration() {
        y = max_lateral_acceleration();
    } else {
        y = missile_frame.y;
    }
    let adjusted = vec2(x, y);
    accelerate(adjusted.rotate(heading()));
}

pub fn turn_to(target_heading: f64) {
    let error = angle_diff(target_heading, heading());
    let time_to_stop = angular_velocity().abs() / max_angular_acceleration();
    let angle_while_stopping =
        angular_velocity() * time_to_stop - 0.5 * max_angular_acceleration() * time_to_stop.powi(2);
    let stopped_error = angle_diff(target_heading, heading() + angle_while_stopping);
    let applied_torque = max_angular_acceleration() * error.signum();
    if stopped_error * error.signum() < 0.0 {
        torque(applied_torque);
    } else {
        torque(-applied_torque);
    }
}

fn angle_at_distance(distance: f64, target_width: f64) -> f64 {
    let sin_theta = target_width / distance;
    sin_theta.asin()
}

