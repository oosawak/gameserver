use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Physics {
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub friction: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            mass: 1.0,
            friction: 0.1,
        }
    }
}

impl Physics {
    pub fn new(mass: f32, friction: f32) -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            mass,
            friction,
        }
    }

    pub fn set_velocity(&mut self, vx: f32, vy: f32) {
        self.vx = vx;
        self.vy = vy;
    }

    pub fn apply_force(&mut self, fx: f32, fy: f32) {
        self.vx += fx / self.mass;
        self.vy += fy / self.mass;
    }

    pub fn speed(&self) -> f32 {
        (self.vx.powi(2) + self.vy.powi(2)).sqrt()
    }
}
