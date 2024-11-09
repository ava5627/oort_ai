use oort_api::prelude::*;

use crate::{target::Target, utils::VecUtils};

pub fn boost_max_acceleration() -> Vec2 {
    if active_abilities().get_ability(Ability::Boost) {
        vec2(
            max_forward_acceleration() + 100.0,
            max_lateral_acceleration(),
        )
    } else {
        vec2(max_forward_acceleration(), max_lateral_acceleration())
    }
}

pub fn best_acceleration(target_heading: f64) -> Vec2 {
    let ma = boost_max_acceleration();
    let angle = ma.angle();
    if angle_diff(heading(), target_heading + angle).abs()
        < angle_diff(heading(), target_heading - angle).abs()
    {
        ma
    } else {
        ma * vec2(1.0, -1.0)
    }
}

pub fn max_accelerate(a: Vec2) {
    let body_frame = a.rotate(-heading());
    let mb = if max_backward_acceleration() > 0.0 {
        -max_backward_acceleration()
    } else {
        0.1
    };
    let x = body_frame.x.clamp(mb, max_forward_acceleration());
    let y = body_frame
        .y
        .clamp(-max_lateral_acceleration(), max_lateral_acceleration());
    let adjusted = vec2(x, y);
    accelerate(adjusted.rotate(heading()));
}

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

pub fn seek(target: &Target) {
    let dp = target.position - position();
    let dv = target.velocity - velocity();
    let closing_speed = -(dp.y * dv.y - dp.x * dv.x).abs() / dp.length();
    let los = dp.angle();
    let los_rate = dv.wedge(dp) / dp.square_magnitude();
    const N: f64 = 4.0;
    let accel = N * closing_speed * los_rate; // + N * nt.length() / 2.0 * los_rate;
    let a = vec2(100.0, accel).rotate(los);
    let target_angle = a.angle();

    let ma = best_acceleration(target_angle);
    let angle = ma.angle();
    if dp.length() > 400.0 {
        let ma = best_acceleration(dp.angle());
        let angle = ma.angle();
        max_accelerate(vec2(ma.x, -ma.y).rotate(target_angle + angle));
        turn_to(a.angle() + angle);
    } else {
        max_accelerate(vec2(ma.x, -ma.y).rotate(dp.angle()));
        turn_to(dp.angle());
    }
    if dp.length() < 100.0 {
        explode();
    }
}
