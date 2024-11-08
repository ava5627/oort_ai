use oort_api::prelude::*;

pub mod debug_utils;
pub mod movement;
pub mod vec_utils;

pub use debug_utils::*;
pub use movement::*;
pub use vec_utils::VecUtils;

use crate::target::Target;

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

pub fn boost(cond: bool, boost_ticks: &mut Option<usize>) {
    if cond && boost_ticks.is_none() {
        activate_ability(Ability::Boost);
        *boost_ticks = Some(0);
    } else if let Some(&ticks) = boost_ticks.as_ref() {
        *boost_ticks = Some(ticks + 1);
        if ticks >= 120 {
            deactivate_ability(Ability::Boost);
        }
        if ticks > 600 {
            *boost_ticks = None;
        }
    }
}

pub fn gun_offsets(gun: usize) -> Vec2 {
    if class() == Class::Fighter || class() == Class::Cruiser {
        vec2(0.0, 0.0)
    } else if class() == Class::Frigate {
        match gun {
            0 => vec2(-40.0, 0.0),
            1 => vec2(0.0, -30.0),
            2 => vec2(0.0, 30.0),
            _ => vec2(0.0, 0.0),
        }
    } else {
        debug!("{:?} does not have a gun", class());
        vec2(0.0, 0.0)
    }
}

pub fn bullet_speeds(gun: usize) -> f64 {
    if class() == Class::Fighter {
        1000.0
    } else if class() == Class::Frigate {
        match gun {
            0 => 4000.0,
            1..=2 => 1000.0,
            _ => 0.0,
        }
    } else if class() == Class::Cruiser {
        2000.0
    } else {
        0.0
    }
}

pub fn lead_target(target: &Target, gun: usize) -> Vec2 {
    let gun_offset = gun_offsets(gun);
    let gun_position = position() - gun_offset.rotate(heading());
    let dp = target.position - gun_position;
    let dv = target.velocity - velocity();

    let bullet_speed = bullet_speeds(gun);
    let time_to_target = dp.length() / bullet_speed;

    let mut future_position = dp
        + dv * time_to_target
        + target.acceleration * time_to_target.powi(2) / 2.0;
    for _ in 0..100 {
        let time_to_target = future_position.length() / bullet_speed;
        let new_future_position = dp
            + dv * time_to_target
            + target.acceleration * time_to_target.powi(2) / 2.0;
        if (future_position - new_future_position).length() < 1e-3 {
            break;
        }
        future_position = new_future_position;
    }
    if future_position.x.is_nan() || future_position.y.is_nan() {
        target.position
    } else {
        future_position
    }
}
