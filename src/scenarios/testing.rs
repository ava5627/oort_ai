
use crate::scenarios::sandbox::kalman_test::KalmanTest;

pub struct Test {
    ship: KalmanTest
}
impl Default for Test {
    fn default() -> Self {
        Self::new()
    }
}

impl Test {
    pub fn new() -> Test {
        Test {
            ship: KalmanTest::new(),
        }
    }
    pub fn tick(&mut self) {
        self.ship.tick();
    }

}
