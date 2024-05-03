use oort_api::prelude::*;
use std::collections::VecDeque;
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
pub fn turn_to_simple(target_heading: f64) {
    let error = angle_diff(heading(), target_heading);
    turn(10.0 * error);
}
pub fn angle_at_distance(distance: f64, target_width: f64) -> f64 {
    let sin_theta = target_width / distance;
    sin_theta.asin()
}
pub fn draw_curve(points: &VecDeque<Vec2>, color: u32, closed: bool) {
    for i in 1..points.len() {
        draw_line(points[i - 1], points[i], color);
    }
    if closed {
        draw_line(points[points.len() - 1], points[0], color);
    }
}
const FIGHTER_SIZE: (f64, f64) = (20.0, 20.0);
const FRIGATE_SIZE: (f64, f64) = (120.0, 50.0);
const CRUISER_SIZE: (f64, f64) = (240.0, 240.0);
pub fn draw_collision_box(class: Class, position: Vec2, rotation: f64) {
    let (width, height) = match class {
        Class::Fighter => FIGHTER_SIZE,
        Class::Frigate => FRIGATE_SIZE,
        Class::Cruiser => CRUISER_SIZE,
        _ => panic!("Invalid class: {:?}", class),
    };
    let corners = vec![
        vec2(-width, -height),
        vec2(width, -height),
        vec2(width, height),
        vec2(-width, height),
    ]
    .into_iter()
    .map(|corner| corner.rotate(rotation) + position)
    .collect::<VecDeque<_>>();
    draw_curve(&corners, 0x00ff00, true);
}
pub fn send_class_and_position() {
    let mut msg = vec![class() as u8];
    msg.extend_from_slice(&[0; 7]);
    msg.extend_from_slice(&position().x.to_le_bytes());
    msg.extend_from_slice(&position().y.to_le_bytes());
    msg.extend_from_slice(&heading().to_le_bytes());
    let checksum = msg.iter().fold(0, |acc: u8, x| acc.wrapping_add(*x));
    msg[7] = checksum;
    send_bytes(&msg)
}
pub fn decode_class_and_position(msg: &[u8]) -> Option<(Class, Vec2, f64)> {
    let checksum = msg[8..32]
        .iter()
        .fold(0, |acc: u8, x| acc.wrapping_add(*x))
        .wrapping_add(msg[0]);
    if checksum != msg[7] {
        debug!("Checksum failed");
        return None;
    }
    let class_u8 = msg[0];
    let class = match class_u8 {
        0 => Class::Fighter,
        1 => Class::Frigate,
        2 => Class::Cruiser,
        _ => Class::Unknown,
    };
    let x = f64::from_le_bytes(msg[8..16].try_into().unwrap());
    let y = f64::from_le_bytes(msg[16..24].try_into().unwrap());
    let position = vec2(x, y);
    let rotation = f64::from_le_bytes(msg[24..32].try_into().unwrap());
    Some((class, position, rotation))
}
