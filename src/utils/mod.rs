use oort_api::prelude::*;

pub mod debug_utils;
pub mod movement;
pub mod vec_utils;

pub use debug_utils::*;
pub use movement::*;
pub use vec_utils::VecUtils;

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
    match class() {
        Class::Fighter => vec2(0.0, 0.0), // Should be -20, 0.0, but for some reason it works better with 0.0
        Class::Frigate => match gun {
            0 => vec2(-40.0, 0.0),
            1 => vec2(0.0, -30.0),
            2 => vec2(0.0, 30.0),
            _ => vec2(0.0, 0.0),
        },
        _ => vec2(0.0, 0.0)
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
    } else if class() == Class::Missile {
        3000.0
    } else {
        0.0
    }
}

pub fn gun_color(gun: usize) -> u32 {
    match gun {
        0 => 0x00ffff,
        1 => 0x00ff00,
        2 => 0xff0000,
        _ => 0xffffff,
    }
}

pub fn class_max_acceleration(class: Class) -> f64 {
    match class {
            Class::Fighter => vec2(160.0, 30.).length(),
            Class::Frigate => vec2(10.0, 5.0).length(),
            Class::Missile => vec2(400.0, 100.0).length(),
            Class::Cruiser => vec2(5.0, 2.5).length(),
            Class::Torpedo => vec2(70.0, 20.0).length(),
            _ => 0.0,
    }
}
