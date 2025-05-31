use oort_api::prelude::*;
pub trait VecUtils {
    fn zero() -> Self;
    fn wedge(&self, other: Self) -> f64;
    fn angle_length(angle: f64, magnitude: f64) -> Self;
    fn square_magnitude(&self) -> f64;
    fn angle_to(&self, other: Self) -> f64;
}
impl VecUtils for Vec2 {
    /// Creates a zero vector.
    fn zero() -> Self {
        vec2(0.0, 0.0)
    }

    /// Calculates the wedge product of two vectors.
    fn wedge(&self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// Calculates the vector from an angle and a magnitude.
    fn angle_length(angle: f64, magnitude: f64) -> Self {
        vec2(magnitude, 0.0).rotate(angle)
    }

    /// Calculates the square of the magnitude of the vector.
    fn square_magnitude(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Calculates the angle from `self` to `other`.
    fn angle_to(&self, other: Self) -> f64 {
        (other - self).angle()
    }
}
