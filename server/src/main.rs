use gameserver_core::{World, Map, GameMode, PlayerInput, gen_entity_id, EntityId};
use std::collections::HashMap;
use tokio::time::{interval, Duration};

mod modes;
use modes::moba::MobaMode;

const TICK_RATE: f64 = 60.0;
const DT: f32 = (1.0 / TICK_RATE) as f32;

#[tokio::main]
async fn main() {
    env_logger::init();

    let map = Map::new("default", 1000.0, 1000.0);
    let mut world = World::new(map);

    let mut game_mode: Box<dyn GameMode> = Box::new(MobaMode::new());
    game_mode.init(&mut world);

    let mut input_queue: HashMap<EntityId, PlayerInput> = HashMap::new();
    let mut interval = interval(Duration::from_millis((1000.0 / TICK_RATE) as u64));

    println!("🎮 Game server started with mode: {}", game_mode.name());
    println!("📊 Tick rate: {} Hz, dt: {:.4}s", TICK_RATE, DT);

    loop {
        interval.tick().await;

        for (_, input) in input_queue.drain() {
            game_mode.handle_input(&mut world, input);
        }

        game_mode.update(&mut world, DT);
        world.step_tick();

        if world.tick % 60 == 0 {
            println!("Tick: {}, Entities: {}", world.tick, world.entities.len());
        }
    }
}
