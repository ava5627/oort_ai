use oort_api::prelude::*;

pub mod cruiser_missile;
pub mod fighter_missile;
pub mod frigate_missile;
pub trait Missile {
    fn new() -> Self;
    fn tick(&mut self);
    fn seek(&mut self);
}

pub fn missile_accelerate(a: Vec2) {
    let missile_frame = a.rotate(-heading());
    let x;
    let y;
    if missile_frame.x < -max_backward_acceleration() {
        x = 0.1;
    } else if missile_frame.x > max_forward_acceleration() {
        x = max_forward_acceleration();
    } else {
        x = missile_frame.x;
    }
    if missile_frame.y < -max_lateral_acceleration() {
        y = -max_lateral_acceleration();
    } else if missile_frame.y > max_lateral_acceleration() {
        y = max_lateral_acceleration();
    } else {
        y = missile_frame.y;
    }
    let adjusted = vec2(x, y);
    accelerate(adjusted.rotate(heading()));
}
fn missile_max_acceleration(boosting: bool) -> Vec2 {
    if boosting {
        vec2(
            max_forward_acceleration() + 100.0,
            max_lateral_acceleration(),
        )
    } else {
        vec2(max_forward_acceleration(), max_lateral_acceleration())
    }
}
