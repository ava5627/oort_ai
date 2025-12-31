pub mod acceleration;
pub mod acceleration2;
pub mod deflection;
pub mod gunnery;
pub mod guns;
pub mod lead;
pub mod missiles;
pub mod radar;
pub mod radio;
pub mod rotation;
pub mod search;
pub mod squad;
pub mod testing;
pub mod sandbox;

use self::testing::Test;
use oort_api::prelude::*;
pub enum Special {
    Test(Test),
    Gun(guns::Ship),
    Acceleration(acceleration::Ship),
    Acceleration2(acceleration2::Ship),
    Rotation(rotation::Ship),
    Lead(lead::Ship),
    Deflection(deflection::Ship),
    Radar(radar::Ship),
    Search(search::Ship),
    Radio(radio::Ship),
    Missiles(missiles::Ship),
    Squad(squad::Ship),
    Gunner(gunnery::Ship),
    None,
}
impl Default for Special {
    fn default() -> Self {
        Self::new()
    }
}

impl Special {
    pub fn new() -> Special {
        match scenario_name() {
            "sandbox" => Special::Test(Test::new()),
            "tutorial_guns" => Special::Gun(guns::Ship::new()),
            "tutorial_acceleration" => Special::Acceleration(acceleration::Ship::new()),
            "tutorial_acceleration2" => Special::Acceleration2(acceleration2::Ship::new()),
            "tutorial_rotation" => Special::Rotation(rotation::Ship::new()),
            "tutorial_lead" => Special::Lead(lead::Ship::new()),
            "tutorial_deflection" => Special::Deflection(deflection::Ship::new()),
            "tutorial_radar" => Special::Radar(radar::Ship::new()),
            "tutorial_search" => Special::Search(search::Ship::new()),
            "tutorial_radio" => Special::Radio(radio::Ship::new()),
            "tutorial_missiles" => Special::Missiles(missiles::Ship::new()),
            "tutorial_squadron" => Special::Squad(squad::Ship::new()),
            "gunnery" => Special::Gunner(gunnery::Ship::new()),
            _ => Special::None,
        }
    }
    pub fn tick(&mut self) {
        match self {
            Special::Test(r) => r.tick(),
            Special::Gun(r) => r.tick(),
            Special::Acceleration(r) => r.tick(),
            Special::Acceleration2(r) => r.tick(),
            Special::Rotation(r) => r.tick(),
            Special::Lead(r) => r.tick(),
            Special::Deflection(r) => r.tick(),
            Special::Radar(r) => r.tick(),
            Special::Search(r) => r.tick(),
            Special::Radio(r) => r.tick(),
            Special::Missiles(r) => r.tick(),
            Special::Squad(r) => r.tick(),
            Special::Gunner(r) => r.tick(),
            Special::None => {}
        }
    }
}
