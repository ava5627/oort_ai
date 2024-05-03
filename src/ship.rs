pub use super::cruiser::Cruiser;
pub use super::fighter::Fighter;
pub use super::frigate::Frigate;
pub use super::missiles::cruiser_missile::CruiserMissile;
pub use super::missiles::fighter_missile::FighterMissile;
pub use super::missiles::frigate_missile::FrigateMissile;
pub use super::missiles::Missile;
pub use super::scenarios::Special;
pub use oort_api::prelude::*;
pub enum Ship {
    Fighter(Fighter),
    FighterMissile(FighterMissile),
    FrigateMissile(FrigateMissile),
    CruiserMissile(CruiserMissile),
    Cruiser(Cruiser),
    Frigate(Frigate),
    Special(Special),
}
impl Default for Ship {
    fn default() -> Self {
        Self::new()
    }
}

impl Ship {
    pub fn new() -> Ship {
        if scenario_name() == "sandbox" {
            return Ship::Special(Special::new());
        }
        match class() {
            Class::Fighter => Ship::Fighter(Fighter::new()),
            Class::Cruiser => Ship::Cruiser(Cruiser::new()),
            Class::Frigate => Ship::Frigate(Frigate::new()),
            Class::Torpedo => Ship::FighterMissile(FighterMissile::new()),
            Class::Missile => match scenario_name() {
                "tutorial_frigate" => Ship::FrigateMissile(FrigateMissile::new()),
                "tutorial_cruiser" => Ship::CruiserMissile(CruiserMissile::new()),
                _ => Ship::FighterMissile(FighterMissile::new()),
            },
            _ => unreachable!(),
        }
    }
    pub fn tick(&mut self) {
        match self {
            Ship::Fighter(f) => f.tick(),
            Ship::FighterMissile(f) => f.tick(),
            Ship::FrigateMissile(f) => f.tick(),
            Ship::CruiserMissile(f) => f.tick(),
            Ship::Cruiser(c) => c.tick(),
            Ship::Frigate(f) => f.tick(),
            Ship::Special(s) => s.tick(),
        }
    }
}
