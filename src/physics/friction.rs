use super::Simulation;
use std::f32::consts::E;

#[derive(Clone, Copy)]
pub struct FrictionDescription {
    pub drag: f32,
}

impl From<f32> for FrictionDescription {
    fn from(value: f32) -> Self {
        Self { drag: value }
    }
}

pub struct FrictionSimulation {
    friction: FrictionDescription,
    drag_log: f32,
    x: f32,
    v: f32,
    tolerance: f32,
}

impl FrictionSimulation {
    pub fn new(
        friction: FrictionDescription,
        position: f32,
        velocity: f32,
        tolerance: f32,
    ) -> Self {
        Self {
            friction,
            drag_log: friction.drag.ln(),
            x: position,
            v: velocity,
            tolerance,
        }
    }

    pub fn through(
        start_position: f32,
        end_position: f32,
        start_velocity: f32,
        end_velocity: f32,
    ) -> Self {
        assert!(
            start_velocity == 0.
                || end_velocity == 0.
                || start_velocity.signum() == end_velocity.signum()
        );
        assert!(start_velocity.abs() >= end_velocity.abs());
        assert!((end_position - start_velocity).signum() == start_velocity.signum());

        let friction =
            Self::drag_for(start_position, end_position, start_velocity, end_velocity).into();
        Self {
            friction,
            drag_log: friction.drag.ln(),
            x: start_position,
            v: start_velocity,
            tolerance: end_velocity.abs(),
        }
    }

    fn drag_for(
        start_position: f32,
        end_position: f32,
        start_velocity: f32,
        end_velocity: f32,
    ) -> f32 {
        E.powf((start_velocity - end_velocity) / (start_position - end_position))
    }

    pub fn final_x(&self) -> f32 {
        self.x - self.v / self.drag_log
    }

    pub fn time_at_x(&self, x: f32) -> f32 {
        if x == self.x {
            0.
        } else if self.v == 0. || {
            if self.v > 0. {
                x < self.x || x > self.final_x()
            } else {
                x > self.x || x < self.final_x()
            }
        } {
            f32::INFINITY
        } else {
            (self.drag_log * (x - self.x) / self.v + 1.).ln() / self.drag_log
        }
    }
}

impl Simulation for FrictionSimulation {
    fn x(&self, time: f32) -> f32 {
        self.x + self.v * self.friction.drag.powf(time) / self.drag_log - self.v / self.drag_log
    }

    fn dx(&self, time: f32) -> f32 {
        self.v * self.friction.drag.powf(time)
    }

    fn is_done(&self, time: f32) -> bool {
        self.dx(time).abs() < self.tolerance
    }
}
