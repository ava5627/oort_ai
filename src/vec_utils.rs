use oort_api::prelude::*;
pub trait VecUtils {
    fn zero() -> Self;
    fn wedge(&self, other: Self) -> f64;
    fn angle_length(angle: f64, magnitude: f64) -> Self;
    fn square_magnitude(&self) -> f64;
    fn angle_to(&self, other: Self) -> f64;
}
impl VecUtils for Vec2 {
    fn zero() -> Self {
        vec2(0.0, 0.0)
    }
    fn wedge(&self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }
    fn angle_length(angle: f64, magnitude: f64) -> Self {
        vec2(magnitude, 0.0).rotate(angle)
    }
    fn square_magnitude(&self) -> f64 {
        self.length().powi(2)
    }
    fn angle_to(&self, other: Self) -> f64 {
        (other - self).angle()
    }
}
