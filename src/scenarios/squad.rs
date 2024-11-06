use oort_api::prelude::*;
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
                send([
                    contact.position.x,
                    contact.position.y,
                    contact.velocity.x,
                    contact.velocity.y,
                ]);
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
                let random_offset = rand(-1.0, 1.0) * TAU / 182.0;
                turn_to(random_offset);
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
    target_position: Vec2,
    last_distance: Vec2,
    target_velocity: Vec2,
    target_acceleration: Vec2,
}
impl Default for Missile {
    fn default() -> Self {
        Self::new()
    }
}

impl Missile {
    pub fn new() -> Missile {
        set_radar_heading(PI);
        Missile {
            target_position: vec2(0.0, 0.0),
            last_distance: vec2(0.0, 0.0),
            target_velocity: vec2(0.0, 0.0),
            target_acceleration: vec2(0.0, 0.0),
        }
    }
    pub fn tick(&mut self) {
        let (target_position, target_velocity) = if let Some(contact) = scan() {
            (contact.position, contact.velocity)
        } else {
            set_radar_heading(radar_heading() + radar_width());
            set_radar_width(TAU / 4.1);
            if let Some(msg) = receive() {
                (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
            } else {
                accelerate(vec2(100.0, 0.0).rotate(heading()));
                return;
            }
        };
        set_radar_heading((target_position - position()).angle());
        set_radar_width(angle_at_distance(
            position().distance(target_position),
            100.0,
        ));
        set_radar_min_distance(position().distance(target_position) - 100.0);
        set_radar_max_distance(position().distance(target_position) + 100.0);
        self.target_acceleration = (target_velocity - self.target_velocity) / TICK_LENGTH;
        self.target_velocity = target_velocity;
        self.target_position = target_position;
        self.seek();
        if angle_diff((self.target_position - position()).angle(), heading()).abs() < 0.5 {
            activate_ability(Ability::Boost);
        }
        if fuel() <= 0.0 {
            set_radar_heading(velocity().angle());
            set_radar_min_distance(0.0);
            set_radar_max_distance(50.0);
            set_radar_width(TAU / 120.0);
        }
    }
    pub fn seek(&mut self) {
        let dp = self.target_position - position();
        let dv = self.target_velocity - velocity();
        let closing_speed = -(dp.y * dv.y - dp.x * dv.x).abs() / dp.length();
        let los = dp.angle();
        let los_rate = (dp.y * dv.x - dp.x * dv.y) / dp.length().powf(2.0);
        const N: f64 = 4.0;
        let nt = self.target_acceleration
            - (self.target_acceleration.dot(dp) / dp.length().powf(2.0)) * dp;
        debug!("nt: {}", nt);
        debug!("nt.length(): {}", nt.length());
        debug!("taccel.length(): {}", self.target_acceleration.length());
        debug!("los_rate: {}", los_rate);
        let accel = N * closing_speed * los_rate + N * nt.length() / 2.0 * los_rate;
        let a = vec2(100.0, accel).rotate(los);
        let a = vec2(400.0, 0.0).rotate(a.angle());
        accelerate(a);
        if dp.length() > 300.0 && fuel() > 0.0 {
            turn_to(a.angle());
        } else {
            turn_to(dp.angle());
        }
        if dp.length() < 100.0 {
            explode();
        }
        debug!("{}", seed());
        self.last_distance = dp;
    }
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
fn angle_at_distance(distance: f64, target_width: f64) -> f64 {
    let sin_theta = target_width / distance;
    sin_theta.asin()
}
