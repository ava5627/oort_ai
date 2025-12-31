use oort_api::prelude::*;

use crate::radar_state::RadarState;
use crate::utils::{angle_at_distance, turn_to_fast, VecUtils};
#[derive(Debug, Clone, PartialEq)]
pub struct TargetState {
    position: Vec2,
    velocity: Vec2,
    last_heading: Option<f64>,
    shots_fired: usize,
    observations: usize,
}
impl TargetState {
    fn load_radar(&self) {
        let dp = self.position - position() + self.velocity * TICK_LENGTH;
        set_radar_heading(dp.angle());
        set_radar_width(angle_at_distance(dp.length(), 20.0));
        set_radar_max_distance(dp.length() + 20.0);
        set_radar_min_distance(dp.length() - 20.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrigateRadarMode {
    FindNewTargets,
    PointDefence,
    UpdateTargets,
}
pub struct Ship {
    targets: Vec<TargetState>,
    current_target: Option<usize>,
    update_index: usize,
    radar_mode: FrigateRadarMode,
    scan_radar: RadarState,
    num_targets: usize,
    fired: bool,
    fp: Option<Vec2>,
    shot_positions: Vec<Vec2>,
}
impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            targets: Vec::new(),
            current_target: None,
            update_index: 0,
            radar_mode: FrigateRadarMode::FindNewTargets,
            scan_radar: RadarState::new(),
            num_targets: 4,
            fired: false,
            fp: None,
            shot_positions: Vec::new(),
        }
    }
    pub fn tick(&mut self) {
        if current_tick() == 0 {
            set_radar_heading(349.0 * PI / 180.0);
        }
        self.update();
        if self.num_targets == 0 {
            torque(max_angular_acceleration());
            set_radar_heading(radar_heading() + radar_width() / 2.0);
            self.scan_radar.set_width(TAU / 4.0);
            return;
        } else if self.radar_mode == FrigateRadarMode::FindNewTargets {
            self.find_targets();
        } else if self.radar_mode == FrigateRadarMode::UpdateTargets {
            self.update_targets();
        }
        if self.targets.len() == 4 {
            if let Some(f) = self.fp {
                draw_triangle(f + position(), 150.0, 0xffffff);
                draw_triangle(f + position(), 10.0, 0xffffff);
            }
        }
        self.aim_and_fire();
        debug!("reload_ticks: {}", reload_ticks(0));
        for p in &self.shot_positions {
            draw_triangle(*p, 50.0, 0xff00ff);
            draw_triangle(*p, 10.0, 0xff00ff);
        }
    }
    fn update(&mut self) {
        if self.num_targets == 0 {
            return;
        }
        for (i, t) in self.targets.iter_mut().enumerate() {
            t.position += t.velocity * TICK_LENGTH;
            draw_polygon(t.position, 50.0, 8, 0.0, 0xffffff);
            draw_square(t.position, 10.0, 0xffffff);
            draw_text!(t.position, 0xffffff, "{:?}", i);
        }
        let mut too_close = None;
        for (i, t) in self.targets.iter().enumerate() {
            let closest_distance = self
                .targets
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(j, other)| (j, t.position.distance(other.position)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            if let Some(closest_distance) = closest_distance.filter(|(_, d)| *d < 10.0) {
                too_close = Some(closest_distance.0);
                break;
            }
        }
        if let Some(idx) = too_close {
            self.targets.remove(idx);
            if self.update_index >= self.targets.len() {
                self.update_index = 0;
            }
        }
    }
    fn find_targets(&mut self) {
        if let Some(contact) = scan() {
            self.new_target(contact.position, contact.velocity);
            set_radar_min_distance(position().distance(contact.position) + 100.0);
            return;
        } else {
            set_radar_min_distance(0.0);
        }
        let mut new_heading = radar_heading() - radar_width() / 2.0;
        let top_angle;
        let bottom_angle;
        if !self.targets.is_empty() {
            let mut max_y = vec2(0.0, -1e9);
            let mut min_y = vec2(0.0, 1e9);
            for t in &self.targets {
                if t.position.y > max_y.y {
                    max_y = t.position;
                }
                if t.position.y < min_y.y {
                    min_y = t.position;
                }
            }
            top_angle = (max_y - position()).angle() + (PI / 50.0);
            bottom_angle = (min_y - position()).angle() - (PI / 50.0);
        } else {
            top_angle = 350.0 * PI / 180.0;
            bottom_angle = 300.0 * PI / 180.0;
        }
        if angle_diff(bottom_angle, new_heading) < 0.0 {
            new_heading = top_angle;
        }
        set_radar_heading(new_heading);
        self.scan_radar.set_width(TAU / 360.0);
        self.scan_radar.save();
        if !self.targets.is_empty() {
            self.targets[0].load_radar();
            self.update_index = 0;
            self.radar_mode = FrigateRadarMode::UpdateTargets;
        } else {
            self.scan_radar.restore();
            self.radar_mode = FrigateRadarMode::FindNewTargets;
        }
    }
    fn new_target(&mut self, new_position: Vec2, new_velocity: Vec2) -> bool {
        for t in &self.targets {
            let distance = (new_position - t.position).length();
            if distance < 20.0 {
                return false;
            }
        }
        let t = TargetState {
            position: new_position,
            velocity: new_velocity,
            last_heading: None,
            shots_fired: 0,
            observations: 1,
        };
        self.targets.push(t);
        self.num_targets = self.targets.len().max(self.num_targets);
        true
    }
    fn aim_and_fire(&mut self) {
        if self.targets.is_empty() {
            self.turn_to_target(-PI / 2.0);
            return;
        } else if !self.fired {
            let fp = if let Some(f) = self.fp {
                if self.targets[0].observations >= 5 {
                    f
                } else {
                    self.predict_turn(self.targets[0].clone())
                }
            } else {
                self.predict_turn(self.targets[0].clone())
            };
            self.fp = Some(fp);
            draw_triangle(fp + position(), 150.0, 0xff0000);
            draw_line(position(), fp + position(), 0xff0000);
            draw_line(
                position(),
                position() + Vec2::angle_length(heading(), fp.length()),
                0x00ff00,
            );
            turn_to_fast((fp).angle() - 0.0005);
            if angle_diff((fp).angle(), heading()).abs() < 0.001 && reload_ticks(0) == 0 {
                fire(0);
                self.targets[0].shots_fired += 1;
                self.num_targets -= 1;
                self.fired = true;
            }
            return;
        }
        let idx = if let Some(t) = self.current_target {
            if t > self.targets.len() - 1 {
                self.current_target = None;
                return;
            }
            t
        } else {
            let index = self
                .targets
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.shots_fired
                        .cmp(&b.shots_fired)
                        .then(a.velocity.partial_cmp(&b.velocity).unwrap().reverse())
                })
                .unwrap()
                .0;
            self.current_target = Some(index);
            index
        };
        debug!("Firing at target {}", idx);
        let target = &self.targets[idx];
        let (target_heading, future_position) =
            lead_target(target.position, target.velocity, 4000.0);
        self.turn_to_target(target_heading);
        let error = angle_diff(target_heading, heading());
        let miss_by = 2.0 * future_position.length() * error.sin();
        if miss_by.abs() < 20.0 && reload_ticks(0) == 0 {
            fire(0);
            self.shot_positions.push(future_position + position());
            self.current_target = None;
            self.targets[idx].shots_fired += 1;
            if self.num_targets > 0 {
                self.num_targets -= 1;
            }
        }
    }
    fn update_targets(&mut self) {
        if let Some(contact) = scan() {
            let target = &mut self.targets[self.update_index];
            target.position = contact.position;
            target.velocity = contact.velocity;
            target.observations += 1;
        } else {
            self.targets.remove(self.update_index);
            if let Some(current_target) = self.current_target {
                match current_target.cmp(&self.update_index) {
                    std::cmp::Ordering::Greater => {
                        self.current_target = Some(current_target - 1);
                    }
                    std::cmp::Ordering::Equal => {
                        self.current_target = None;
                    }
                    std::cmp::Ordering::Less => {}
                }
            }
            self.update_index -= 1;
        }
        if self.update_index < self.targets.len()
            && self.targets[self.update_index].observations < 7
        {
            self.targets[self.update_index].load_radar();
        } else if self.update_index + 1 < self.targets.len() {
            self.update_index += 1;
            self.targets[self.update_index].load_radar();
        } else {
            self.scan_radar.restore();
            self.radar_mode = FrigateRadarMode::FindNewTargets;
        }
    }
    fn turn_to_target(&mut self, target_heading: f64) {
        let error = angle_diff(target_heading, heading());
        let last_heading = match self.current_target {
            Some(current_target) => self.targets[current_target].last_heading,
            None => None,
        };
        let tav_angular_velocity = match last_heading {
            Some(last_target_heading) => (target_heading - last_target_heading) / TICK_LENGTH,
            None => 0.0,
        };
        if let Some(current_target) = self.current_target {
            self.targets[current_target].last_heading = Some(target_heading);
        }
        let time_to_stop =
            (angular_velocity() - tav_angular_velocity).abs() / max_angular_acceleration();
        let angle_while_stopping = (angular_velocity() - tav_angular_velocity) * time_to_stop
            - 0.5 * max_angular_acceleration() * time_to_stop.powi(2) * error.signum();
        let target_when_stopped = target_heading + time_to_stop * tav_angular_velocity;
        let stopped_error = angle_diff(target_when_stopped, heading() + angle_while_stopping);
        let applied_torque = max_angular_acceleration() * error.signum();
        if stopped_error * error.signum() < 0.0 {
            torque(applied_torque);
        } else {
            torque(-applied_torque);
        }
    }
    fn predict_turn(&self, target: TargetState) -> Vec2 {
        let dp = target.position - position();
        let dv = target.velocity - velocity();
        let time_to_target = dp.length() / 4000.0;
        let mut future_position = dp + dv * time_to_target;
        for _ in 0..200 {
            let turn_time = time_to_turn_to(future_position.angle());
            let time_to_target = (future_position.length() - 40.0) / 4000.0;
            let new_future_position = dp + dv * (time_to_target + turn_time);
            let delta = new_future_position.distance(future_position);
            future_position = new_future_position;
            if delta < 1e-3 {
                break;
            }
        }
        future_position
    }
}

fn time_to_turn_to(target_heading: f64) -> f64 {
    let av = angular_velocity() * TICK_LENGTH;
    let curr_error = angle_diff(target_heading, heading());
    let aa = max_angular_acceleration() * TICK_LENGTH * TICK_LENGTH;

    let accel_sign = curr_error.signum() * -1.0;
    let passed = ((-(aa / 2.0 + av)
        + ((aa / 2.0 + av).powi(2) + 2.0 * aa * curr_error.abs()).sqrt() * accel_sign)
        / aa)
        .ceil()
        .abs();
    passed * TICK_LENGTH
}
fn lead_target(target_position: Vec2, target_velocity: Vec2, bullet_speed: f64) -> (f64, Vec2) {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    let time_to_target = dp.length() / bullet_speed;
    let mut future_position = dp + dv * time_to_target;
    for _ in 0..1000 {
        let time_to_target = (future_position.length() - 40.0) / bullet_speed;
        let new_future_position = dp + dv * time_to_target;
        let delta = new_future_position.distance(future_position);
        future_position = new_future_position;
        if delta < 1e-3 {
            break;
        }
    }
    let real_future_position = future_position + position();
    draw_triangle(real_future_position, 10.0, 0xffffff);
    draw_triangle(real_future_position, 100.0, 0xffffff);
    draw_line(position(), real_future_position, 0xffffff);
    let actual_target = vec2(future_position.length() * 100., 0.0).rotate(heading()) + position();
    draw_line(position(), actual_target, 0x00ff00);
    draw_triangle(actual_target, 100.0, 0x00ff00);
    (future_position.angle(), future_position)
}
