pub mod friction;
pub mod spring;

pub const DEFAULT_TOLERANCE: f32 = 0.001;

fn near_equal(a: f32, b: f32, epsilon: f32) -> bool {
    (a > (b - epsilon)) && (a < (b + epsilon)) || a == b
}

fn near_zero(n: f32, epsilon: f32) -> bool {
    near_equal(n, 0., epsilon)
}

pub trait Simulation {
    fn x(&self, time: f32) -> f32;
    fn dx(&self, time: f32) -> f32;
    fn is_done(&self, time: f32) -> bool;
}
