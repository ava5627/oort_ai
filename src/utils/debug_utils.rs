use oort_api::prelude::*;
use std::collections::VecDeque;

const FIGHTER_SIZE: (f64, f64) = (20.0, 20.0);
const FRIGATE_SIZE: (f64, f64) = (120.0, 50.0);
const CRUISER_SIZE: (f64, f64) = (240.0, 240.0);

pub fn draw_curve(points: &VecDeque<Vec2>, color: u32, closed: bool) {
    points.iter().fold(None, |prev, point| {
        if let Some(prev) = prev {
            draw_line(prev, *point, color);
        }
        Some(*point)
    });
    if closed {
        draw_line(points[points.len() - 1], points[0], color);
    }
}

pub fn draw_points(points: &VecDeque<(Vec2, u32)>, color: u32) {
    for (point, time) in points {
        if current_tick() > *time {
            continue; // Skip points older than 100 ticks
        } else if current_tick() == *time {
            draw_square(*point, 10.0, 0xffffff); // Draw old points in white
        } else {
            draw_square(*point, 10.0, color);
            // draw_text!(
            //     *point + vec2(0.0, 15.0),
            //     0xffffff,
            //     "{}",
            //     time - current_tick(),
            // );
        }
    }
}

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

pub fn draw_heading(distance: f64) {
    draw_line(
        position(),
        position() + vec2(distance, 0.0).rotate(heading()),
        0x00ff00,
    );
}
