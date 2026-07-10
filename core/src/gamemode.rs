use crate::world::World;
use crate::input::PlayerInput;

pub trait GameMode: Send + Sync {
    fn init(&mut self, world: &mut World);

    fn handle_input(&mut self, world: &mut World, input: PlayerInput);

    fn update(&mut self, world: &mut World, dt: f32);

    fn name(&self) -> &'static str;
}
