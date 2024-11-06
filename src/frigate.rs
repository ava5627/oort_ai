use std::cmp::Ordering;

use crate::pid::PID;
use crate::radar_state::RadarState;
use crate::target::Target;
use oort_api::prelude::*;
const TURRET_BULLET_SPEED: f64 = 1000.0;
const MAIN_BULLET_SPEED: f64 = 4000.0;
#[derive(PartialEq)]
enum TargetHeuristic {
    Angle,
    Distance,
    Other,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrigateRadarMode {
    FindNewTargets,
    UpdateTargets,
}
pub struct Frigate {
    targets: Vec<Target>,
    index: usize,
    radar_mode: FrigateRadarMode,
    scan_radar: RadarState,
    pid: PID,
}
impl Default for Frigate {
    fn default() -> Self {
        Self::new()
    }
}

impl Frigate {
    pub fn new() -> Frigate {
        set_radar_heading(PI);
        Frigate {
            targets: Vec::new(),
            index: 0,
            radar_mode: FrigateRadarMode::FindNewTargets,
            scan_radar: RadarState::new(),
            pid: PID::new(
                8.0,
                0.0,
                300.0 / 60.0,
                max_angular_acceleration(),
                max_angular_acceleration(),
            ),
        }
    }
    pub fn tick(&mut self) {
        debug!("targets {:?}", self.targets.len());
        debug!("index {:?}", self.index);
        debug!("radar_mode {:?}", self.radar_mode);
        if reload_ticks(3) == 0 {
            fire(3);
        }
        if self.radar_mode == FrigateRadarMode::FindNewTargets {
            self.find_targets();
        } else if self.radar_mode == FrigateRadarMode::UpdateTargets {
            self.update_targets();
        }
        self.fire_turrets();
        for (i, t) in self.targets.iter_mut().enumerate() {
            t.tick();
            draw_polygon(t.position, 50.0, 8, 0.0, 0xffffff);
            draw_text!(t.position, 0xffffff, "{:?}", i);
        }
    }
    fn find_targets(&mut self) {
        if let Some(contact) = scan() {
            self.new_target(contact.position, contact.velocity, contact.class);
        }
        set_radar_heading(radar_heading() + radar_width());
        set_radar_width(TAU / 20.0);
        self.scan_radar.save();
        if !self.targets.is_empty() {
            self.targets[0].load_radar();
            self.index = 0;
            self.radar_mode = FrigateRadarMode::UpdateTargets;
        } else {
            self.scan_radar.restore();
            self.radar_mode = FrigateRadarMode::FindNewTargets;
        }
    }
    fn update_targets(&mut self) {
        if let Some(contact) = scan() {
            if contact.class == Class::Missile || contact.class == Class::Torpedo {
                if self.index + 1 < self.targets.len() {
                    self.index += 1;
                    self.targets[self.index].load_radar();
                } else {
                    self.scan_radar.restore();
                    self.radar_mode = FrigateRadarMode::FindNewTargets;
                }
                return;
            }
            let target = &mut self.targets[self.index];
            if !target.sanity_check(contact.position, contact.velocity) {
                debug!("target failed sanity check");
                let new_target = self.targets.iter_mut().enumerate().min_by(|a, b| {
                    a.1.position
                        .distance(contact.position)
                        .partial_cmp(&b.1.position.distance(contact.position))
                        .unwrap()
                });
                if let Some((i, new_target)) = new_target {
                    if i != self.index {
                        debug!("switching to new target");
                        new_target.update(contact.position, contact.velocity);
                    }
                }
            } else {
                target.update(contact.position, contact.velocity);
            }
        } else {
            debug!("lost target");
            self.targets.remove(self.index);
            self.index -= 1;
        }
        if self.index + 1 < self.targets.len() {
            self.index += 1;
            self.targets[self.index].load_radar();
        } else {
            self.scan_radar.restore();
            self.radar_mode = FrigateRadarMode::FindNewTargets;
        }
    }
    fn new_target(&mut self, new_position: Vec2, new_velocity: Vec2, new_class: Class) {
        if new_class == Class::Missile || new_class == Class::Torpedo {
            return;
        }
        for t in &self.targets {
            if t.class == new_class && t.position.distance(new_position) < 100.0 {
                return;
            }
        }
        let t = Target::new(new_position, new_velocity, new_class);
        self.targets.push(t);
    }
    fn fire_turrets(&mut self) {
        let mut free_targets = self
            .targets
            .iter()
            .enumerate()
            .filter(|(_, t)| t.shots_fired == 0)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        for weapon_idx in [0, 1, 2, 3] {
            if free_targets.is_empty() {
                break;
            }
            let mut t_index = free_targets[weapon_idx % free_targets.len()];
            if weapon_idx == 0 {
                t_index = self.prioritize_targets(weapon_idx, &free_targets);
                if free_targets.len() > 2 {
                    free_targets.retain(|&x| x != t_index);
                }
            }
            let prediction = self.lead_target(t_index, weapon_idx);
            let target = &mut self.targets[t_index];
            let angle = prediction.angle();
            if weapon_idx == 0 {
                debug!(
                    "Main weapon targeting {}, reloded in {}",
                    t_index,
                    reload_ticks(weapon_idx)
                );
                let miss_by = angle_diff(heading(), angle) * prediction.length();
                let applied_torque = self.pid.update(angle_diff(heading(), angle));
                torque(applied_torque);
                if miss_by.abs() < 7.0 && reload_ticks(weapon_idx) == 0 {
                    fire(weapon_idx);
                    target.shots_fired += 1;
                }
            } else if weapon_idx == 3 {
                debug!(
                    "Missiles targeting {}, reloaded in {}",
                    t_index,
                    reload_ticks(weapon_idx)
                );
                send([
                    target.position.x,
                    target.position.y,
                    target.velocity.x,
                    target.velocity.y,
                ]);
            } else {
                debug!(
                    "Turret {} targeting {}, reloded in {}",
                    weapon_idx,
                    t_index,
                    reload_ticks(weapon_idx)
                );
                aim(weapon_idx, angle);
                fire(weapon_idx);
            }
        }
    }
    fn prioritize_targets(&self, weapon_index: usize, free_targets: &[usize]) -> usize {
        let heuristic = match weapon_index {
            0 => (TargetHeuristic::Angle, true),
            1 => (TargetHeuristic::Angle, false),
            2 => (TargetHeuristic::Distance, true),
            3 => (TargetHeuristic::Other, false),
            _ => unreachable!(),
        };
        if heuristic.0 == TargetHeuristic::Other {
            return free_targets[0];
        }
        *free_targets
            .iter()
            .min_by(|&&a, &&b| {
                let a = &self.targets[a];
                let b = &self.targets[b];
                self.evaluate(a, b, &heuristic)
            })
            .unwrap()
    }
    fn evaluate(
        &self,
        target: &Target,
        other: &Target,
        heuristic: &(TargetHeuristic, bool),
    ) -> Ordering {
        let cmp = match heuristic.0 {
            TargetHeuristic::Angle => {
                let angle = angle_diff(heading(), (target.position - position()).angle()).abs();
                let other_angle =
                    angle_diff(heading(), (other.position - position()).angle()).abs();
                angle.partial_cmp(&other_angle).unwrap()
            }
            TargetHeuristic::Distance => {
                let distance = position().distance(target.position);
                let other_distance = position().distance(other.position);
                distance.partial_cmp(&other_distance).unwrap()
            }
            TargetHeuristic::Other => {
                return Ordering::Equal;
            }
        };
        if heuristic.1 {
            cmp
        } else {
            cmp.reverse()
        }
    }
    fn lead_target(&mut self, target_index: usize, gun: usize) -> Vec2 {
        let targets_len = self.targets.len() as f64;
        let target = &mut self.targets[target_index];
        let bullet_speed = if gun == 0 {
            MAIN_BULLET_SPEED
        } else {
            TURRET_BULLET_SPEED
        };
        let p = if gun == 1 {
            position() - vec2(0.0, -30.0).rotate(heading())
        } else if gun == 2 {
            position() - vec2(0.0, 30.0).rotate(heading())
        } else if gun == 0 {
            position() - vec2(-40.0, 0.0).rotate(heading())
        } else {
            return vec2(0.0, 0.0);
        };
        let dp = target.position - p;
        let dv = target.velocity - velocity();
        let mut time_to_target = dp.length() / bullet_speed;
        let jerk =
            (target.acceleration - target.last_acceleration) / (TICK_LENGTH * (targets_len + 1.0));
        let mut future_position = dp
            + dv * time_to_target
            + target.acceleration * time_to_target.powi(2) / 2.0
            + jerk * time_to_target.powi(3) / 6.0;
        for _ in 0..100 {
            time_to_target = future_position.length() / bullet_speed;
            let new_future_position = dp
                + dv * time_to_target
                + target.acceleration * time_to_target.powi(2) / 2.0
                + jerk * time_to_target.powi(3) / 6.0;
            let delta = new_future_position - future_position;
            future_position = new_future_position;
            if delta.length() < 1e-3 {
                break;
            }
        }
        let color = if gun == 0 {
            let distance = future_position.length();
            draw_line(p, vec2(distance, 0.0).rotate(heading()) + p, 0xffff00);
            draw_triangle(vec2(distance, 0.0).rotate(heading()) + p, 10.0, 0xffff00);
            0x00ffff
        } else if gun == 1 {
            0x00ff00
        } else {
            0xff0000
        };
        draw_polygon(future_position + p, 10.0, 4, 0.0, color);
        draw_line(p, future_position + p, color);
        future_position
    }
}
