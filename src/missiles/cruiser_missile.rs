use crate::missiles::Missile;
use crate::target::Target;
use crate::utils::VecUtils;
use crate::utils::{angle_at_distance, boost, max_accelerate, turn_to};
use oort_api::prelude::*;
pub struct CruiserMissile {
    target: Option<Target>,
    boost_time: Option<usize>,
}
impl Missile for CruiserMissile {
    fn new() -> CruiserMissile {
        let radio_channel = id() % 8;
        set_radio_channel(radio_channel as usize);
        CruiserMissile {
            target: None,
            boost_time: None,
        }
    }
    fn tick(&mut self) {
        debug!("id {:?}", id());
        debug!("radio_channel {:?}", get_radio_channel());
        let (target_position, target_velocity) = if let Some(contact) = scan() {
            if contact.class == Class::Missile || contact.class == Class::Torpedo {
                set_radar_heading(radar_heading() + contact.position.angle());
                set_radar_width(TAU / 8.0);
                if let Some(msg) = receive() {
                    (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
                } else {
                    accelerate(vec2(100.0, 0.0).rotate(heading()));
                    return;
                }
            } else {
                (contact.position, contact.velocity)
            }
        } else if let Some(msg) = receive() {
            debug!("received on {} {:?}", get_radio_channel(), msg);
            (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
        } else {
            let radio_channel = get_radio_channel();
            set_radio_channel((radio_channel + 1) % 8);
            debug!("radio_channel {:?}", radio_channel);
            set_radar_heading(radar_heading() + radar_width() * position().y.signum());
            set_radar_width(TAU / 4.0);
            turn_to(0.0);
            accelerate(vec2(200.0, 0.0));
            return;
        };
        if class() != Class::Torpedo
            && target_position.y.signum() != position().y.signum()
            && target_position.y.abs() > 30.0
            && position().x.abs() < 40.0
        {
            debug!("Target behind cruiser");
            let radio_channel = get_radio_channel();
            set_radio_channel((radio_channel + 1) % 8);
            set_radar_heading(radar_heading() + radar_width() * position().y.signum());
            set_radar_width(TAU / 4.0);
            let accel = vec2(200.0, 0.0) * target_position.x.signum();
            accelerate(accel);
            turn_to(accel.angle());
            return;
        }
        set_radar_heading((target_position - position()).angle());
        set_radar_width(angle_at_distance(
            position().distance(target_position),
            100.0,
        ));
        set_radar_max_distance(position().distance(target_position) + 50.0);
        set_radar_min_distance(position().distance(target_position) - 50.0);
        if let Some(target) = &mut self.target {
            if target.position.distance(target_position) < 100.0 {
                target.update(target_position, target_velocity);
            } else {
                self.target = Some(Target::new(
                    target_position,
                    target_velocity,
                    Class::Missile,
                ));
            }
        } else {
            self.target = Some(Target::new(
                target_position,
                target_velocity,
                Class::Missile,
            ));
        }
        self.seek();
        debug!(
            "{:?}",
            angle_diff((target_position - position()).angle(), heading()).abs()
        );
        boost(
            angle_diff((target_position - position()).angle(), heading()).abs() < PI / 4.0,
            &mut self.boost_time,
        );
    }

    fn seek(&mut self) {
        let target = self.target.as_ref().unwrap();
        let dp = target.position - position();
        let dv = target.velocity - velocity();
        let closing_speed = -(dp.y * dv.y - dp.x * dv.x).abs() / dp.length();
        let los = dp.angle();
        let los_rate = dv.wedge(dp) / dp.square_magnitude();
        const N: f64 = 4.0;
        let accel = N * closing_speed * los_rate; // + N * nt.length() / 2.0 * los_rate;
        let a = vec2(100.0, accel).rotate(los);
        let a = Vec2::angle_length(a.angle(), 400.0);
        let boosting = self.boost_time.map_or(true, |t| t < 120);
        let b2 = active_abilities().get_ability(Ability::Boost);
        debug!("boosting2 {:?}", b2);
        debug!("boosting  {:?}", boosting);
        let ma = boost_max_acceleration(b2);
        debug!("ma  {:?}", ma);
        let ma = best_acceleration(b2, a.angle());
        debug!("ma2 {:?}", ma);
        let angle = ma.angle();
        let target_angle = a.angle();
        accelerate(a);
        if dp.length() > 400.0 {
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
}
pub fn boost_max_acceleration(boosting: bool) -> Vec2 {
    if boosting {
        vec2(
            max_forward_acceleration() + 100.0,
            max_lateral_acceleration(),
        )
    } else {
        vec2(max_forward_acceleration(), max_lateral_acceleration())
    }
}

pub fn best_acceleration(boosting: bool, target_heading: f64) -> Vec2 {
    let ma = boost_max_acceleration(boosting);
    let angle = ma.angle();
    if angle_diff(heading(), target_heading + angle).abs()
        < angle_diff(heading(), target_heading - angle).abs()
    {
        ma
    } else {
        ma * vec2(1.0, -1.0)
    }
}
