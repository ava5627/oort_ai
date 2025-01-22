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
        let (target_position, target_velocity) = if let Some(contact) =
            scan().filter(|c| ![Class::Missile, Class::Torpedo].contains(&c.class))
        {
            (contact.position, contact.velocity)
        } else if let Some(msg) = receive() {
            (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
        } else {
            no_target();
            turn_to(0.0);
            accelerate(vec2(200.0, 0.0));
            return;
        };
        if class() != Class::Torpedo
            && target_position.y.signum() != position().y.signum()
            && target_position.y.abs() > 30.0
            && position().x.abs() < 100.0
        {
            debug!("target behind cruiser");
            no_target();
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
        if let Some(target) = self
            .target
            .as_mut()
            .filter(|t| t.sanity_check(target_position, target_velocity, Class::Missile))
        {
            target.update(target_position, target_velocity);
        } else {
            self.target = Some(Target::new(
                target_position,
                target_velocity,
                Class::Missile,
            ));
        }
        let target = self.target.as_ref().unwrap();
        let dp = target.position - position();
        if dp.length() > 500.0 {
            seek(target);
        } else {
            final_approach(target);
        }
        let dv = target.velocity - velocity();
        debug!("dp {:>8.3}", dp.length());
        debug!("dv {:>8.3}", dv.length());
        if dp.length() < 130.0 {
            explode();
        }
        let error = angle_diff(dp.angle(), heading()).abs();
        let should_boost = error < PI / 4.0;
        boost(should_boost, &mut self.boost_time);
    }
}

fn no_target() {
    let radio_channel = get_radio_channel();
    set_radio_channel((radio_channel + 1) % 8);
    set_radar_heading(radar_heading() + radar_width() * position().y.signum());
    set_radar_width(TAU / 4.0);
    set_radar_max_distance(10000.0);
    set_radar_min_distance(0.0);
}
