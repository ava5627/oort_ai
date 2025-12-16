use std::cmp::Ordering;

use crate::pid::PID;
use crate::radar_state::RadarState;
use crate::target::Target;
use maths_rs::num::Cast;
use oort_api::prelude::*;
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
    found_all_targets: bool,
}
impl Default for Frigate {
    fn default() -> Self {
        Self::new()
    }
}

impl Frigate {
    pub fn new() -> Frigate {
        Frigate {
            targets: Vec::new(),
            index: 0,
            radar_mode: FrigateRadarMode::FindNewTargets,
            scan_radar: RadarState::new(),
            pid: PID::new(
                12.0,
                0.0,
                6.0,
                max_angular_acceleration(),
                max_angular_acceleration(),
            ),
            found_all_targets: false,
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
            t.tick(i);
        }
    }
    fn find_targets(&mut self) {
        if let Some(contact) = scan() {
            self.new_target(contact.position, contact.velocity, contact.class);
            set_radar_min_distance(contact.position.distance(position()) + 20.0);
        } else {
            self.scan_radar.rotate();
        }
        self.scan_radar.save();
        if self.targets.len() >= 5 {
            self.found_all_targets = true;
        }
        if self.found_all_targets {
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
            if !target.sanity_check(contact.position, contact.velocity, contact.class) {
                debug!("target failed sanity check");
                let new_target = self
                    .targets
                    .iter_mut()
                    .enumerate()
                    .min_by_key(|a| a.1.position.distance(contact.position).as_i64());
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
        } else if !self.targets.is_empty() {
            self.index = 0;
            self.targets[0].load_radar();
        } else {
            self.found_all_targets = false;
            self.scan_radar.restore();
            self.radar_mode = FrigateRadarMode::FindNewTargets;
        }
    }
    fn new_target(&mut self, new_position: Vec2, new_velocity: Vec2, new_class: Class) {
        if new_class == Class::Missile || new_class == Class::Torpedo {
            return;
        }
        for t in &self.targets {
            if t.sanity_check(new_position, new_velocity, new_class) {
                return;
            }
        }
        let t = Target::new(new_position, new_velocity, new_class);
        self.targets.push(t);
    }
    fn fire_turrets(&mut self) {
        if self.targets.is_empty() {
            return;
        }
        let min_shots = self
            .targets
            .iter()
            .map(|t| t.shots_fired)
            .min()
            .unwrap_or(0);
        let mut free_targets = self
            .targets
            .iter()
            .enumerate()
            .filter(|(_, t)| t.shots_fired == min_shots)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        for weapon_idx in [0, 1, 2, 3] {
            let mut t_index = free_targets[weapon_idx % free_targets.len()];
            if weapon_idx == 0 {
                t_index = self.prioritize_targets(weapon_idx, &free_targets);
                if free_targets.len() > 2 {
                    free_targets.retain(|&x| x != t_index);
                }
            }
            let target = &mut self.targets[t_index];
            if weapon_idx == 0 {
                debug!(
                    "Main weapon targeting {}, reloded in {}",
                    t_index,
                    reload_ticks(weapon_idx)
                );
                let prediction = target.lead(weapon_idx);
                let angle = prediction.angle();
                let miss_by = angle_diff(heading(), angle) * prediction.length();
                let applied_torque = self.pid.update(angle_diff(heading(), angle));
                torque(applied_torque);
                if miss_by.abs() < 7.0 && reload_ticks(weapon_idx) == 0 {
                    fire(weapon_idx);
                    self.pid.reset();
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
                let prediction = target.lead(weapon_idx);
                let angle = prediction.angle();
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
        match heuristic.1 {
            true => cmp,
            false => cmp.reverse(),
        }
    }
}
