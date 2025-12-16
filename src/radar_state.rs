use oort_api::prelude::*;
#[derive(Debug, PartialEq)]
pub struct RadarState {
    heading: f64,
    width: f64,
    min_distance: f64,
    max_distance: f64,
    turns: usize,
    rotations: usize,
}
impl RadarState {
    pub fn new() -> RadarState {
        RadarState {
            heading: PI/2.0,
            width: PI / 2.0,
            min_distance: 0.0,
            max_distance: 1e99,
            turns: 1,
            rotations: 0,
        }
    }

    pub fn rotate(&mut self) {
        set_radar_min_distance(0.0);
        set_radar_heading(self.heading + self.width);
        if self.rotations > self.turns * 4 {
            self.rotations = 0;
            self.turns += 1;
            self.width /= 2.0;
        }
        self.rotations += 1;
    }
    pub fn save(&mut self) {
        self.heading = radar_heading();
        // self.width = radar_width();
        self.min_distance = radar_min_distance();
        self.max_distance = radar_max_distance();
    }
    pub fn restore(&self) {
        set_radar_heading(self.heading);
        set_radar_width(self.width);
        set_radar_min_distance(self.min_distance);
        set_radar_max_distance(self.max_distance);
    }
}

impl Default for RadarState {
    fn default() -> Self {
        Self::new()
    }
}
