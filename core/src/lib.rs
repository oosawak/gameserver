pub mod entity;
pub mod world;
pub mod physics;
pub mod map;
pub mod gamemode;
pub mod input;

pub use entity::{Entity, EntityId, EntityKind, Transform, Health};
pub use world::World;
pub use physics::Physics;
pub use map::Map;
pub use gamemode::GameMode;
pub use input::PlayerInput;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static ENTITY_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn gen_entity_id() -> EntityId {
    EntityId(ENTITY_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
}
