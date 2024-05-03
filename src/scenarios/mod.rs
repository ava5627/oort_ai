pub mod acceleration;
pub mod acceleration2;
pub mod deflection;
pub mod gunnery;
pub mod lead;
pub mod missiles;
pub mod radio;
pub mod rotation;
pub mod search;
pub mod squad;
pub mod testing;

use self::testing::Test;
use oort_api::prelude::*;
pub enum Special {
    Test(Test),
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
            _ => unreachable!(),
        }
    }
    pub fn tick(&mut self) {
        match self {
            Special::Test(r) => r.tick(),
        }
    }
}
