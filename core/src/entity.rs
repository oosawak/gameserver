use serde::{Deserialize, Serialize};
use crate::physics::Physics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale: f32,
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            rotation: 0.0,
            scale: 1.0,
        }
    }

    pub fn distance_to(&self, other: &Transform) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
        }
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Player,
    Ball,
    Projectile,
    Obstacle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,
    pub transform: Transform,
    pub physics: Physics,
    pub health: Option<Health>,
    pub owner: Option<EntityId>,
    pub radius: f32,
}

impl Entity {
    pub fn new(id: EntityId, kind: EntityKind, transform: Transform, radius: f32) -> Self {
        Self {
            id,
            kind,
            transform,
            physics: Physics::default(),
            health: None,
            owner: None,
            radius,
        }
    }

    pub fn with_health(mut self, health: Health) -> Self {
        self.health = Some(health);
        self
    }

    pub fn with_owner(mut self, owner: EntityId) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn update_physics(&mut self, dt: f32) {
        self.transform.x += self.physics.vx * dt;
        self.transform.y += self.physics.vy * dt;

        self.physics.vx *= 1.0 - self.physics.friction * dt;
        self.physics.vy *= 1.0 - self.physics.friction * dt;
    }
}
