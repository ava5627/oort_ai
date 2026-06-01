use oort_api::prelude::*;

use crate::radar_state::RadarState;
use crate::utils::{angle_at_distance, turn_to, turn_to_no_stop, VecUtils};
#[derive(Debug, Clone, PartialEq)]
pub struct TargetState {
    position: Vec2,
    velocity: Vec2,
    last_heading: Option<f64>,
    predicted_position: Option<Vec2>,
    shots_fired: usize,
    observations: usize,
    last_shot_position: Option<Vec2>,
}
impl TargetState {
    fn load_radar(&self) {
        let dp = self.position - position() + self.velocity * TICK_LENGTH;
        set_radar_heading(dp.angle());
        set_radar_width(angle_at_distance(dp.length(), 20.0));
        set_radar_max_distance(dp.length() + 20.0);
        set_radar_min_distance(dp.length() - 20.0);
    }

    fn draw(&self, index: usize) {
        draw_polygon(self.position, 50.0, 8, 0.0, 0xffffff);
        draw_square(self.position, 10.0, 0xffffff);
        draw_text!(self.position, 0xffffff, "{:?}", index);
        if let Some(last_shot_position) = self.last_shot_position {
            draw_triangle(last_shot_position, 50.0, 0xff0000);
            draw_triangle(last_shot_position, 10.0, 0xff0000);
            draw_line(last_shot_position, self.position, 0xff0000);
            draw_line(position(), last_shot_position, 0xff0000);
        }
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
            scan_radar: RadarState::default(),
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
        if self.radar_mode == FrigateRadarMode::FindNewTargets {
            self.find_targets();
        } else if self.radar_mode == FrigateRadarMode::UpdateTargets {
            self.update_targets();
        }
        self.aim_and_fire();
        debug!("reload_ticks: {}", reload_ticks(0));
    }
    fn update(&mut self) {
        for (i, t) in self.targets.iter_mut().enumerate() {
            t.position += t.velocity * TICK_LENGTH;
            t.draw(i);
            t.predicted_position = Some(lead_target(t.position, t.velocity, 4001.0).1);
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
        self.scan_radar.set_width(TAU / 300.0);
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
            predicted_position: None,
            last_shot_position: None,
        };
        self.targets.push(t);
        true
    }
    fn aim_and_fire(&mut self) {
        if self.targets.is_empty() {
            turn_to(-PI / 2.0);
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
            draw_triangle(fp + position(), 10.0, 0xff0000);
            draw_line(position(), fp + position(), 0xff0000);
            draw_line(
                position(),
                position() + Vec2::angle_length(heading(), fp.length()),
                0x00ff00,
            );
            let mut double_shot = None;
            for (i, t) in self.targets.iter().enumerate() {
                let x = t.position.x;
                // find the y of the bullet at the x of the target
                let angle = angle_diff((fp).angle(), 0.0);
                let bullet_y = position().y - (position().x - x).abs() * angle.tan();
                // find time to reach the x of the target
                let turn_time = if angle_diff((fp).angle(), heading()) < 0.001 {
                    0.0
                } else {
                    time_to_turn_to((fp).angle())
                };
                let time_to_x = vec2(x, bullet_y).distance(position()) / 4000.0 + turn_time;
                // find the y of the target at that time
                let target_position = t.position + t.velocity * time_to_x;
                let dist = target_position.distance(fp + position());
                let error = (bullet_y - target_position.y).abs();

                let impact_position = (fp).rotate(-0.0005).normalize()
                    * target_position.distance(position())
                    + position();
                let deflect_up = Vec2::angle_length((fp).angle() + 0.1, dist);
                let deflect_down = Vec2::angle_length((fp).angle() - 0.1, dist);
                let deflect_range = deflect_up.distance(deflect_down);
                let hit_chance = 20.0 / deflect_range;
                // if the bullet is within 20 units of the target, draw a green line,
                if error < 10.0 {
                    double_shot = Some((i, impact_position));
                    if i != 0 {
                        debug!(
                            "Target {} is within range for a double shot! (error: {:.3}, dist: {:.2})",
                            i, error, dist
                        );
                        draw_polygon(target_position, 20.0, 6, 0.0, 0x00ff00);
                        debug!("Deflect range for target {}: {:.3}", i, deflect_range);
                        debug!("Hit chance for target {}: {:.2}%", i, hit_chance * 100.0);
                        draw_line(impact_position, impact_position + deflect_up, 0x00ff00);
                        draw_line(impact_position, impact_position + deflect_down, 0x00ff00);
                        draw_polygon(fp + position(), 20.0, 4, 0.0, 0x00ffff);
                    }
                }
            }
            let offset = if let Some((i, _)) = double_shot {
                if i != 0 {
                    debug!("Double shot at target {}", i);
                }
                -0.0005
            } else {
                0.0005
            };
            turn_to_no_stop((fp).angle() + offset);
            if angle_diff((fp).angle(), heading()).abs() < 0.001 && reload_ticks(0) == 0 {
                fire(0);
                self.targets[0].shots_fired += 2;
                self.targets[0].last_shot_position = Some(fp + position());
                self.fired = true;
                if let Some((double_shot, shot_position)) = double_shot {
                    self.targets[double_shot].shots_fired += 1;
                    self.targets[double_shot].last_shot_position = Some(shot_position);
                }
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
                    a.shots_fired.cmp(&b.shots_fired).then(
                        time_to_turn_to(a.predicted_position.unwrap().angle())
                            .partial_cmp(&time_to_turn_to(b.predicted_position.unwrap().angle()))
                            .unwrap(),
                    )
                })
                .unwrap()
                .0;
            self.current_target = Some(index);
            index
        };
        debug!("Firing at target {}", idx);
        let target = &mut self.targets[idx];
        let (target_heading, future_position) =
            lead_target(target.position, target.velocity, 4000.0);
        target.predicted_position = Some(future_position + position());
        let actual_target = vec2(future_position.length(), 0.0).rotate(heading()) + position();
        draw_polygon(future_position + position(), 100.0, 6, 0.0, 0xffffff);
        draw_polygon(future_position + position(), 10.0, 6, 0.0, 0xffffff);
        draw_triangle(actual_target, 100.0, 0x00ff00);
        draw_line(position(), actual_target, 0x00ff00);
        draw_line(position(), future_position + position(), 0xffffff);
        turn_to_target(target);
        let error = angle_diff(target_heading, heading());
        let miss_by = 2.0 * future_position.length() * error.sin();
        if miss_by.abs() < 20.0 && reload_ticks(0) == 0 {
            fire(0);
            self.shot_positions.push(future_position + position());
            self.current_target = None;
            target.last_shot_position = Some(future_position + position());
            target.shots_fired += 2;
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
fn turn_to_target(target: &mut TargetState) {
    if target.predicted_position.is_none() {
        return;
    }
    let target_heading = (target.predicted_position.unwrap() - position()).angle();
    let last_heading = target.last_heading.unwrap_or(target_heading);
    target.last_heading = Some(target_heading);
    let delta_heading = angle_diff(target_heading, last_heading);
    let av = angular_velocity() * TICK_LENGTH + delta_heading;
    let aa = max_angular_acceleration() * TICK_LENGTH * TICK_LENGTH;
    let time_to_stop = av.abs() / aa;
    let heading_when_stopped = heading() + av * time_to_stop
        - 0.5 * aa * av.signum() * (time_to_stop.powi(2) + time_to_stop);
    let av_next_tick = av + aa * av.signum();
    let heading_next_tick = heading() + av_next_tick - aa * av.signum();
    let time_to_stop_later = av_next_tick.abs() / aa;
    let heading_when_stopped_later = heading_next_tick + av_next_tick * time_to_stop_later
        - 0.5 * aa * (time_to_stop_later.powi(2) + time_to_stop_later) * av_next_tick.signum();
    let error = angle_diff(target_heading, heading_when_stopped);
    let error_later = angle_diff(target_heading, heading_when_stopped_later);
    let error_per_tick =
        (error * 2.0 / (time_to_stop.powi(2) + time_to_stop)) / TICK_LENGTH / TICK_LENGTH;
    let accel = if error.signum() != error_later.signum() {
        max_angular_acceleration() * error.signum() - error_per_tick
    } else {
        -max_angular_acceleration() * error.signum()
    };
    torque(accel);
}

fn time_to_turn_to(target_heading: f64) -> f64 {
    let av = angular_velocity() * TICK_LENGTH;
    let curr_error = angle_diff(heading(), target_heading);
    let accel_sign = curr_error.signum();
    let aa = max_angular_acceleration() * TICK_LENGTH * TICK_LENGTH * accel_sign;

    let passed = ((-(aa / 2.0 + av)
        + ((aa / 2.0 + av).powi(2) + 2.0 * aa * curr_error).sqrt() * accel_sign)
        / aa)
        .ceil();
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
    (future_position.angle(), future_position)
}
