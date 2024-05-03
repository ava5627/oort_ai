use oort_api::prelude::*;
#[derive(Debug, PartialEq)]
pub struct RadarState {
    heading: f64,
    width: f64,
    min_distance: f64,
    max_distance: f64,
}
impl RadarState {
    pub fn new() -> RadarState {
        RadarState {
            heading: 0.0,
            width: TAU / 120.0,
            min_distance: 0.0,
            max_distance: 1e99,
        }
    }
    pub fn save(&mut self) {
        self.heading = radar_heading();
        self.width = radar_width();
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
