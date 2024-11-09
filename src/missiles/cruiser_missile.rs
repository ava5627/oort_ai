use crate::missiles::Missile;
use crate::target::Target;
use crate::utils::angle_at_distance;
use crate::utils::final_approach;
use crate::utils::{boost, seek, turn_to};
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
        if let Some(target) = &self.target {
            let dp = target.position - position();
            if dp.length() > 500.0 {
                seek(target);
            } else {
                final_approach(target);
            }
            if dp.length() < 130.0 {
                explode();
            }
            let error = angle_diff(dp.angle(), heading()).abs();
            let should_boost = error < PI / 4.0;
            boost(should_boost, &mut self.boost_time);
        }
    }

    fn seek(&mut self) {}
}
