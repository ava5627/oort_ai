use crate::radar_state::RadarState;
use crate::target::{Target, TentativeTarget};
use crate::utils::{angle_at_distance, send_class_and_position};
use crate::utils::VecUtils;
use oort_api::prelude::*;
const TURRET_BULLET_SPEED: f64 = 2000.0;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CruiserRadarMode {
    FindNewTargets,
    PointDefence,
    UpdateTargets,
}
pub struct Cruiser {
    scan_radar: RadarState,
    targets: Vec<Target>,
    tentative_target: TentativeTarget,
    index: usize,
    radar_mode: CruiserRadarMode,
}
impl Default for Cruiser {
    fn default() -> Self {
        Self::new()
    }
}

impl Cruiser {
    pub fn new() -> Cruiser {
        Cruiser {
            scan_radar: RadarState::new(),
            targets: Vec::new(),
            tentative_target: TentativeTarget {
                positions: Vec::new(),
                average_position: Vec2::zero(),
                class: Class::Unknown,
            },
            index: 0,
            radar_mode: CruiserRadarMode::FindNewTargets,
        }
    }
    pub fn tick(&mut self) {
        select_radio(7);
        set_radio_channel(9);
        send_class_and_position();
        debug!("targets {:?}", self.targets.len());
        debug!("index {:?}", self.index);
        debug!("radar_mode {:?}", self.radar_mode);
        fire(1);
        fire(2);
        fire(3);
        debug!("gun reload {:?}", reload_ticks(0));
        debug!("port missile reload {:?}", reload_ticks(1));
        debug!("starboard missile reload {:?}", reload_ticks(2));
        debug!("torpedo reload {:?}", reload_ticks(3));
        if self.radar_mode == CruiserRadarMode::FindNewTargets {
            self.find_targets();
        } else if self.radar_mode == CruiserRadarMode::UpdateTargets {
            self.update_targets();
        }
        if !self.targets.is_empty() {
            let furthest_index = self
                .targets
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    position()
                        .distance(a.position)
                        .partial_cmp(&position().distance(b.position))
                        .unwrap()
                })
                .unwrap_or((0, &self.targets[0]))
                .0;
            for i in 0..8 {
                let mut index = i % self.targets.len();
                if index == furthest_index && reload_ticks(0) < 100 {
                    index = (index + 1) % self.targets.len();
                }
                let t = &self.targets[index];
                select_radio(i);
                set_radio_channel(i);
                send([t.position.x, t.position.y, t.velocity.x, t.velocity.y]);
            }
            let angle = self.lead_target(furthest_index, TURRET_BULLET_SPEED);
            aim(0, angle);
            fire(0);
        }
        for (i, t) in self.targets.iter_mut().enumerate() {
            t.tick();
            draw_polygon(t.position, 50.0, 8, 0.0, 0xffffff);
            draw_text!(t.position, 0xffffff, "{:?}", i);
        }
    }
    fn find_targets(&mut self) {
        if let Some(contact) = scan() {
            debug!("contact snr {:?}", contact.snr);
            if contact.snr < 10.0 {
                self.tentative_target.positions.push(contact.position);
                let average_position = self
                    .tentative_target
                    .positions
                    .iter()
                    .fold(Vec2::zero(), |acc, t| acc + t)
                    / self.tentative_target.positions.len() as f64;
                if self.tentative_target.positions.len() > 10 {
                    self.tentative_target.positions.remove(0);
                }
                self.tentative_target.average_position = average_position;
                set_radar_heading((average_position - position()).angle());
                set_radar_width(angle_at_distance(
                    position().distance(average_position),
                    1000.0,
                ));
                return;
            }
            self.new_target(contact.position, contact.velocity, contact.class);
        }
        set_radar_heading(radar_heading() + radar_width());
        set_radar_width(TAU / 10.0);
        self.scan_radar.save();
        if !self.targets.is_empty() {
            self.index = (self.index + 1) % self.targets.len();
            self.targets[self.index].load_radar();
            self.radar_mode = CruiserRadarMode::UpdateTargets;
        } else {
            self.scan_radar.restore();
            self.radar_mode = CruiserRadarMode::FindNewTargets;
        }
    }
    fn update_targets(&mut self) {
        if let Some(contact) = scan() {
            set_radar_heading(contact.position.angle());
            set_radar_width(angle_at_distance(
                position().distance(contact.position),
                100.0,
            ));
            set_radar_max_distance(contact.position.length() + 100.0);
            set_radar_min_distance(contact.position.length() - 100.0);
            let target = &mut self.targets[self.index];
            target.update(contact.position, contact.velocity);
        } else {
            debug!("lost target");
            self.targets.remove(self.index);
            self.index -= 1;
        }
        self.scan_radar.restore();
        self.radar_mode = CruiserRadarMode::FindNewTargets;
    }
    fn new_target(&mut self, new_position: Vec2, new_velocity: Vec2, new_class: Class) {
        if new_class == Class::Missile || new_class == Class::Torpedo {
            return;
        }
        for t in &self.targets {
            let distance = t.position.distance(new_position);
            if t.class == new_class && distance < 200.0 {
                return;
            }
        }
        let target = Target::new(new_position, new_velocity, new_class);
        self.targets.push(target);
    }
    fn lead_target(&self, target_index: usize, bullet_speed: f64) -> f64 {
        let target = &self.targets[target_index];
        let dp = target.position - position();
        let dv = target.velocity - velocity();
        let time_to_target = dp.length() / bullet_speed;
        let mut future_position =
            dp + dv * time_to_target + target.acceleration * time_to_target.powf(2.0) / 2.0;
        for _ in 0..100 {
            let time_to_target = future_position.length() / bullet_speed;
            let new_future_position =
                dp + dv * time_to_target + target.acceleration * time_to_target.powf(2.0) / 2.0;
            let delta = new_future_position - future_position;
            if delta.length() < 1e-3 {
                break;
            }
            future_position = new_future_position;
        }
        let color = 0x00ff00;
        draw_polygon(future_position, 10.0, 4, 0.0, color);
        (future_position - position()).angle()
    }
}
