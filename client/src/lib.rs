use wasm_bindgen::prelude::*;
use gameserver_core::{World, Map, Entity, EntityKind, Transform, gen_entity_id};

#[wasm_bindgen]
pub struct GameClient {
    world: World,
    predicted_entities: Vec<Entity>,
    local_tick: u64,
}

#[wasm_bindgen]
impl GameClient {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f32, height: f32) -> GameClient {
        let map = Map::new("default", width, height);
        let world = World::new(map);

        GameClient {
            world,
            predicted_entities: Vec::new(),
            local_tick: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.local_tick += 1;

        for entity in self.world.entities.values_mut() {
            entity.update_physics(dt);

            let (clamped_x, clamped_y) = self.world.map.clamp_to_bounds(
                entity.transform.x,
                entity.transform.y,
                entity.radius,
            );
            entity.transform.x = clamped_x;
            entity.transform.y = clamped_y;
        }
    }

    pub fn apply_local_prediction(&mut self, move_x: f32, move_y: f32) {
        for entity in self.world.entities.values_mut() {
            if entity.kind == EntityKind::Player {
                entity.physics.vx = move_x * 200.0;
                entity.physics.vy = move_y * 200.0;
                break;
            }
        }
    }

    pub fn reconcile_from_server(&mut self, server_tick: u64, server_state: &str) {
        self.local_tick = server_tick;
    }

    pub fn get_entity_count(&self) -> usize {
        self.world.entities.len()
    }

    pub fn get_tick(&self) -> u64 {
        self.world.tick
    }
}
