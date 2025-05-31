use oort_api::prelude::*;

use crate::utils::{angle_at_distance, VecUtils};

const BEARING_NOISE_FACTOR: f64 = 1e1 * (TAU / 360.0);
const DISTANCE_NOISE_FACTOR: f64 = 1e4;
const VELOCITY_NOISE_FACTOR: f64 = 1e2;
const MAX_MEASUREMENTS: usize = 100;

#[derive(Debug)]
pub struct KalmanFilter {
    measurements: Vec<(Vec2, Vec2, f64, Vec2, Vec2)>, // (position, velocity, snr, my_position, my_velocity)
    pub predicted_position: Vec2,
    distance_variance: f64,
    bearing_variance: f64,
}

impl Default for KalmanFilter {
    fn default() -> Self {
        Self::new()
    }
}
impl KalmanFilter {
    pub fn new() -> KalmanFilter {
        KalmanFilter {
            measurements: Vec::new(),
            predicted_position: Vec2::zero(),
            distance_variance: 0.0,
            bearing_variance: 0.0,
        }
    }

    pub fn add_measurement(
        &mut self,
        target_position: Vec2,
        target_velocity: Vec2,
        snr: f64,
    ) {
        self.measurements
            .push((target_position, target_velocity, snr, position(), velocity()));
        if self.measurements.len() > MAX_MEASUREMENTS {
            self.measurements.remove(0); // Keep the most recent measurements
        }
    }

    pub fn run(&mut self) {
        let mut distance_mean = self
            .measurements
            .iter()
            .map(|(tp, _, _, mp, _)| mp.distance(*tp))
            .sum::<f64>()
            / self.measurements.len() as f64;
        let mut distance_variance = 1e6; // Initial high new_variance
        let mut bearing_mean = self
            .measurements
            .iter()
            .map(|(tp, _, _, mp, _)| mp.angle_to(*tp))
            .sum::<f64>()
            / self.measurements.len() as f64;
        let mut bearing_variance = 1e6; // Initial high new_variance

        // Iterate through measurements and update Kalman filter
        for (target_position, target_velocity, snr, my_p, my_v) in &self.measurements {
            let error_factor = 10.0f64.powf(-snr / 10.0);
            let distance_variance_m = (error_factor * DISTANCE_NOISE_FACTOR).powi(2);
            let bearing_variance_m = (error_factor * BEARING_NOISE_FACTOR).powi(2);
            let velocity_variance_m = (error_factor * VELOCITY_NOISE_FACTOR).powi(2);
            let distance = my_p.distance(*target_position);
            let bearing = my_p.angle_to(*target_position);

            // Update Kalman filter for distance
            (distance_mean, distance_variance) = update_kalman(
                distance_mean,
                distance_variance,
                distance,
                distance_variance_m,
            );

            // Update Kalman filter for bearing
            (bearing_mean, bearing_variance) =
                update_kalman(bearing_mean, bearing_variance, bearing, bearing_variance_m);

            let my_future_position = my_p + *my_v * TICK_LENGTH;
            let target_future_position = *target_position + *target_velocity * TICK_LENGTH;
            let distance_movement = my_future_position.distance(target_future_position) - distance;

            let bearing_movement =
                angle_diff(bearing, my_future_position.angle_to(target_future_position));

            // Predict next state
            (distance_mean, distance_variance) = predict_kalman(
                distance_mean,
                distance_variance,
                distance_movement,
                velocity_variance_m,
            );
            (bearing_mean, bearing_variance) = predict_kalman(
                bearing_mean,
                bearing_variance,
                bearing_movement,
                bearing_variance_m,
            );
        }
        let uncertainty_width = distance_variance.powi(2);
        let latest_contact_velocity = self
            .measurements
            .last()
            .map_or(Vec2::zero(), |(_, v, _, _, _)| *v);
        let predicted_position = Vec2::angle_length(bearing_mean, distance_mean) + position() + (velocity() - latest_contact_velocity) * TICK_LENGTH;
        // let future_position = predicted_position + (velocity() - latest_contact_velocity) * TICK_LENGTH;
        draw_polygon(predicted_position, uncertainty_width, 6, 0.0, 0xffffff);
        draw_polygon(predicted_position, 5.0, 6, 0.0, 0xffff00);

        self.predicted_position = predicted_position;
        self.distance_variance = distance_variance;
        self.bearing_variance = bearing_variance;
    }

    pub fn point_radar(&self) {
        let distance = position().distance(self.predicted_position);
        set_radar_heading(position().angle_to(self.predicted_position));
        let width = 25.0 * (1.0 + self.bearing_variance.sqrt());
        set_radar_width(angle_at_distance(distance, width));
        let height = 25.0 * (1.0 + self.distance_variance.sqrt());
        set_radar_max_distance(distance + height);
        set_radar_min_distance(distance - height);
    }

    pub fn reset(&mut self) {
        self.measurements.clear();
        self.predicted_position = Vec2::zero();
        self.distance_variance = 0.0;
        self.bearing_variance = 0.0;
    }
}

pub fn position_to_bearing_distance(target_position: Vec2) -> (f64, f64) {
    (
        position().angle_to(target_position),
        position().distance(target_position),
    )
}

fn update_kalman(
    mean: f64,
    variance: f64,
    measurement: f64,
    measurement_variance: f64,
) -> (f64, f64) {
    let new_mean =
        (mean * measurement_variance + measurement * variance) / (variance + measurement_variance);
    let new_variance = 1.0 / (1.0 / variance + 1.0 / measurement_variance);
    (new_mean, new_variance)
}

fn predict_kalman(mean: f64, variance: f64, movement: f64, movement_variance: f64) -> (f64, f64) {
    let new_mean = mean + movement;
    let new_variance = variance + movement_variance;
    (new_mean, new_variance)
}
