use serde::{Deserialize, Serialize};
use crate::entity::EntityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub player_id: EntityId,
    pub move_x: f32,
    pub move_y: f32,
    pub action1: bool,
    pub action2: bool,
    pub tick: u64,
}

impl PlayerInput {
    pub fn new(player_id: EntityId, tick: u64) -> Self {
        Self {
            player_id,
            move_x: 0.0,
            move_y: 0.0,
            action1: false,
            action2: false,
            tick,
        }
    }

    pub fn with_move(mut self, move_x: f32, move_y: f32) -> Self {
        self.move_x = move_x;
        self.move_y = move_y;
        self
    }

    pub fn with_action1(mut self, action: bool) -> Self {
        self.action1 = action;
        self
    }

    pub fn with_action2(mut self, action: bool) -> Self {
        self.action2 = action;
        self
    }
}
