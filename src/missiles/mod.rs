pub mod cruiser_missile;
pub mod frigate_missile;
pub mod fighter_missile;
pub trait Missile {
    fn new() -> Self;
    fn tick(&mut self);
    fn seek(&mut self);
}
