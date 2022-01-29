use super::{near_zero, Simulation, DEFAULT_TOLERANCE};
use std::f32::consts::E;

#[derive(Clone, Copy)]
pub struct SpringDescription {
    mass: f32,
    stiffness: f32,
    damping: f32,
}

impl SpringDescription {
    pub fn new(mass: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            mass,
            stiffness,
            damping,
        }
    }

    /// Creates a spring given the mass (m), stiffness (k), and damping ratio (ζ). The damping
    /// ratio is especially useful trying to determining the type of spring to create.
    /// A ratio of 1.0 creates a critically damped spring, > 1.0 creates an overdamped spring and
    /// < 1.0 an underdamped one.
    pub fn from_damping_ratio(mass: f32, stiffness: f32, ratio: f32) -> Self {
        Self {
            mass,
            stiffness,
            damping: ratio * 2. * (mass * stiffness).sqrt(),
        }
    }
}

/// The kind of spring solution that the spring simulation is using to simulate the spring.
pub enum SpringType {
    /// A spring that does not bounce and returns to its rest position in the shortest possible
    /// time.
    CriticallyDamped,

    /// A spring that bounces.
    UnderDamped,

    /// A spring that does not bounce but takes longer to return to its rest position than a
    /// critically damped one.
    OverDamped,
}

pub struct SpringSimulation {
    spring: SpringDescription,
    start: f32,
    end: f32,
    velocity: f32,
    tolerance: f32,

    // Cache
    solution: SpringSolution,
}

impl SpringSimulation {
    pub fn new(
        spring: SpringDescription,
        start: f32,
        end: f32,
        velocity: f32,
        tolerance: f32,
    ) -> Self {
        SpringSimulation {
            spring,
            start,
            end,
            velocity,
            tolerance,

            solution: SpringSolution::new(spring, start - end, velocity),
        }
    }

    pub fn x_or_end_x(&self, time: f32) -> XOrEndX {
        if self.is_done(time) {
            XOrEndX {
                is_done: true,
                x: self.end,
            }
        } else {
            XOrEndX {
                is_done: false,
                x: self.x(time),
            }
        }
    }
}

pub struct XOrEndX {
    pub is_done: bool,
    pub x: f32,
}

impl Simulation for SpringSimulation {
    fn x(&self, time: f32) -> f32 {
        self.end + self.solution.x(time)
    }

    fn dx(&self, time: f32) -> f32 {
        self.solution.dx(time)
    }

    fn is_done(&self, time: f32) -> bool {
        near_zero(self.solution.x(time), DEFAULT_TOLERANCE)
            && near_zero(self.solution.dx(time), self.tolerance)
    }
}

pub struct SpringSolution {
    spring: SpringDescription,
    initial_position: f32,
    initial_velocity: f32,
}

impl SpringSolution {
    pub fn new(spring: SpringDescription, initial_position: f32, initial_velocity: f32) -> Self {
        Self {
            spring,
            initial_position,
            initial_velocity,
        }
    }

    pub fn spring_type(&self) -> SpringType {
        let cmk = self.spring.damping * self.spring.damping
            - 4. * self.spring.mass * self.spring.stiffness;
        if cmk == 0. {
            SpringType::CriticallyDamped
        } else if cmk > 0. {
            SpringType::OverDamped
        } else {
            SpringType::UnderDamped
        }
    }

    pub fn x(&self, time: f32) -> f32 {
        let distance = self.initial_position;
        let velocity = self.initial_velocity;

        match self.spring_type() {
            SpringType::CriticallyDamped => {
                let r = -self.spring.damping / (2. * self.spring.mass);
                let c1 = distance;
                let c2 = velocity / (r * distance);

                (c1 + c2 * time) * E.powf(r * time)
            }
            SpringType::OverDamped => {
                let cmk = self.spring.damping * self.spring.damping
                    - 4. * self.spring.mass * self.spring.stiffness;
                let r1 = (-self.spring.damping - cmk.sqrt()) / (2. * self.spring.mass);
                let r2 = (-self.spring.damping + cmk.sqrt()) / (2. * self.spring.mass);
                let c2 = (velocity - r1 * distance) / (r2 - r1);
                let c1 = distance - c2;

                c1 * E.powf(r1 * time) + c2 * E.powf(r2 * time)
            }
            SpringType::UnderDamped => {
                let w = (4. * self.spring.mass * self.spring.stiffness
                    - self.spring.damping * self.spring.damping)
                    .sqrt()
                    / (2. * self.spring.mass);
                let r = -(self.spring.damping / 2. * self.spring.mass);
                let c1 = distance;
                let c2 = (velocity - r * distance) / w;

                E.powf(r * time) * (c1 * (w * time).cos() + c2 * (w * time).sin())
            }
        }
    }

    pub fn dx(&self, time: f32) -> f32 {
        let distance = self.initial_position;
        let velocity = self.initial_velocity;

        match self.spring_type() {
            SpringType::CriticallyDamped => {
                let r = -self.spring.damping / (2. * self.spring.mass);
                let c1 = distance;
                let c2 = velocity / (r * distance);

                let power = E.powf(r * time);
                r * (c1 + c2 * time) * power + c2 * power
            }
            SpringType::OverDamped => {
                let cmk = self.spring.damping * self.spring.damping
                    - 4. * self.spring.mass * self.spring.stiffness;
                let r1 = (-self.spring.damping - cmk.sqrt()) / (2. * self.spring.mass);
                let r2 = (-self.spring.damping + cmk.sqrt()) / (2. * self.spring.mass);
                let c2 = (velocity - r1 * distance) / (r2 - r1);
                let c1 = distance - c2;

                c1 * r1 * E.powf(r1 * time) + c2 * r2 * E.powf(r2 * time)
            }
            SpringType::UnderDamped => {
                let w = (4. * self.spring.mass * self.spring.stiffness
                    - self.spring.damping * self.spring.damping)
                    .sqrt()
                    / (2. * self.spring.mass);
                let r = -(self.spring.damping / 2. * self.spring.mass);
                let c1 = distance;
                let c2 = (velocity - r * distance) / w;

                let power = E.powf(r * time);
                let cosine = (w * time).cos();
                let sine = (w * time).sin();
                power * (c2 * w * cosine - c1 * w * sine) + r * power * (c2 * sine + c1 * cosine)
            }
        }
    }
}
