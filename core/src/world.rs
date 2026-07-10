use std::collections::HashMap;
use crate::entity::{Entity, EntityId};
use crate::map::Map;

pub struct World {
    pub entities: HashMap<EntityId, Entity>,
    pub map: Map,
    pub tick: u64,
}

impl World {
    pub fn new(map: Map) -> Self {
        Self {
            entities: HashMap::new(),
            map,
            tick: 0,
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> EntityId {
        let id = entity.id;
        self.entities.insert(id, entity);
        id
    }

    pub fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    pub fn get_entities_by_owner(&self, owner: EntityId) -> Vec<&Entity> {
        self.entities.values().filter(|e| e.owner == Some(owner)).collect()
    }

    pub fn step_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }
}
