use oort_api::prelude::*;

use crate::target::Target;
use crate::utils::angle_at_distance;
use crate::utils::best_acceleration;
use crate::utils::boost;
use crate::utils::max_accelerate;
use crate::utils::turn_to;
use crate::utils::turn_to_simple;
const BULLET_SPEED: f64 = 1000.0;
pub struct Ship {
    fighter: Fighter,
    missile: Missile,
}
impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            fighter: Fighter::new(),
            missile: Missile::new(),
        }
    }
    pub fn tick(&mut self) {
        set_radio_channel(0);
        match class() {
            Class::Fighter => {
                self.fighter.tick();
            }
            Class::Missile => {
                self.missile.tick();
            }
            _ => {}
        }
    }
}
pub struct Fighter {
    last_vel: Vec2,
}
impl Default for Fighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Fighter {
    pub fn new() -> Fighter {
        Fighter {
            last_vel: vec2(0.0, 0.0),
        }
    }
    pub fn tick(&mut self) {
        activate_ability(Ability::Boost);
        if let Some(contact) = scan() {
            if contact.class == Class::Missile {
                set_radar_min_distance((contact.position - position()).length());
                accelerate(vec2(100.0, 0.0));
                fire(0);
                fire(1);
                return;
            }
            if self.last_vel.length() == 0.0 {
                debug!("first contact");
                self.last_vel = contact.velocity - velocity();
            }
            set_radar_min_distance((contact.position - position()).length() - 1000.0);
            set_radar_max_distance((contact.position - position()).length() + 1000.0);
            set_radar_width(angle_at_distance(
                (contact.position - position()).length(),
                100.0,
            ));
            fire(1);
            fire(0);
            send([
                contact.position.x,
                contact.position.y,
                contact.velocity.x,
                contact.velocity.y,
            ]);
            let dp = contact.position - position();
            draw_line(position(), contact.position, 0xffffff);
            set_radar_heading(dp.angle());
            set_radar_width((10.0 * TAU / dp.length()).clamp(TAU / 30.0, TAU));
            draw_line(
                position(),
                vec2(dp.length(), 0.0).rotate(dp.angle()),
                0xff00ff,
            );
            if dp.length() < 2000.0 || contact.snr > 35.0 {
                let future_target = lead_target(contact.position, contact.velocity, self.last_vel);
                draw_line(position(), future_target, 0x00ff00);
                draw_line(
                    position(),
                    vec2(future_target.distance(position()), 0.0).rotate(heading()),
                    0x00ff00,
                );
                draw_triangle(future_target + position(), 100.0, 0x00ff00);
                let angle = future_target.angle();
                turn_to(angle);
            } else {
                let random_offset = rand(-1.0, 1.0) * TAU / 162.0;
                turn_to_simple(random_offset);
                accelerate(contact.position);
            }
            self.last_vel = contact.velocity - velocity();
        } else {
            set_radar_heading(radar_heading() + radar_width());
            set_radar_width(TAU / 60.0);
            let random_offset = rand(-1.0, 1.0) * TAU / 40.0;
            turn_to(random_offset);
            accelerate(vec2(100.0, 0.0));
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
        Missile {
            target: None,
            boost_time: None,
        }
    }
    pub fn tick(&mut self) {
        debug!("position: {:?}", position());
        let (target_position, target_velocity) = if let Some(contact) =
            scan().filter(|c| c.class != Class::Missile && self.target.is_some())
        {
            (contact.position, contact.velocity)
        // } else if let Some(msg) = receive() {
        //     debug!("received message");
        //     (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
        } else if let Some(contact) = scan().filter(|c| c.class != Class::Missile) {
            (contact.position, contact.velocity)
        } else if fuel() > 0.0 {
            set_radar_heading(0.0);
            // let d = position().x.abs() * 2.0;
            // let a = angle_at_distance(d, 1000.0);
            if position().x < -200.0 {
                set_radar_width(TAU / 128.0);
            } else {
                set_radar_width(TAU / 64.0);
            }
            set_radar_max_distance(1e99);
            set_radar_min_distance(0.0);
            accelerate(vec2(100.0, 0.0));
            return;
        } else {
            set_radar_heading(radar_heading() + radar_width());
            set_radar_width(TAU / 4.0);
            set_radar_max_distance(1e99);
            set_radar_min_distance(0.0);
            return;
        };
        set_radar_heading((target_position - position()).angle());
        set_radar_width(angle_at_distance(
            position().distance(target_position),
            200.0,
        ));
        set_radar_min_distance(position().distance(target_position) - 200.0);
        set_radar_max_distance(position().distance(target_position) + 200.0);
        if let Some(target) = &mut self.target {
            if target_position.distance(target.position) < 200.0 {
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
        self.seek_target();
        if fuel() <= 0.0 {
            set_radar_heading(velocity().angle());
            set_radar_min_distance(0.0);
            set_radar_max_distance(1000.0);
            set_radar_width(TAU / 20.0);
        }
    }
    pub fn seek_target(&mut self) {
        let target = if let Some(target) = &self.target {
            target
        } else {
            return;
        };
        let dp = target.position - position();
        let dv = target.velocity - velocity();
        let closing_speed = -(dp.y * dv.y - dp.x * dv.x).abs() / dp.length();
        let los = dp.angle();
        let los_rate = (dp.y * dv.x - dp.x * dv.y) / dp.length().powf(2.0);
        const N: f64 = 4.0;
        let nt = target.acceleration - (target.acceleration.dot(dp) / dp.length().powf(2.0)) * dp;
        let accel = N * closing_speed * los_rate + N * nt.length() / 2.0 * los_rate;
        let a = vec2(100.0, accel).rotate(los);
        let target_angle = a.angle();
        let ma = best_acceleration(dp.angle());
        let angle = ma.angle();
        max_accelerate(vec2(ma.x, -ma.y).rotate(target_angle + angle));
        turn_to(target_angle + angle);
        if dp.length() > 300.0 && fuel() > 0.0 {
            turn_to_simple(a.angle());
        } else {
            turn_to_simple(dp.angle());
        }
        if dp.length() < 300.0 {
            explode();
        }
        let error = angle_diff(heading(), dp.angle()).abs();
        let should_boost = error < 2.0 && fuel() > 0.0;
        boost(should_boost, &mut self.boost_time);
    }
}

fn lead_target(target_position: Vec2, target_velocity: Vec2, last_vel: Vec2) -> Vec2 {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    let da = dv - last_vel;
    let time_to_target = dp.length() / BULLET_SPEED;
    let mut future_position = dp + dv * time_to_target + da * time_to_target.powi(2) / 2.0;
    let mut delta = 1e9;
    for _ in 0..1000 {
        let time_to_target = future_position.length() / BULLET_SPEED;
        let new_pos = dp + dv * time_to_target + da * time_to_target.powi(2) / 2.0;
        if (future_position - new_pos).length() > delta {
            break;
        }
        delta = (future_position - new_pos).length();
        future_position = new_pos;
        if delta < 1e-3 {
            break;
        }
    }
    future_position
}
