use gameserver_core::{World, Map, GameMode, PlayerInput, gen_entity_id, EntityId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tokio_tungstenite::accept_async;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::env;
use tokio::net::TcpListener;

mod moba;
use moba::MobaMode;

const TICK_RATE: f64 = 60.0;
const DT: f32 = (1.0 / TICK_RATE) as f32;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameState {
    tick: u64,
    players: Vec<PlayerState>,
    entities: Vec<EntityState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlayerState {
    id: u64,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    health: f32,
    max_health: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EntityState {
    id: u64,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    kind: String,
    health: Option<f32>,
    max_health: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "join")]
    Join { name: String },
    #[serde(rename = "input")]
    Input {
        move_x: f32,
        move_y: f32,
        action1: bool,
    },
    #[serde(rename = "clear_participants")]
    ClearParticipants,
    #[serde(rename = "set_speed")]
    SetSpeed { speed: f32 },
    #[serde(rename = "set_auto_move")]
    SetAutoMove { entity_id: u64, enabled: bool },
    #[serde(rename = "leave")]
    Leave,
}

struct GameServer {
    world: World,
    mode: Box<dyn GameMode>,
    client_players: HashMap<usize, EntityId>,
    auto_move_entities: HashMap<EntityId, AutoMoveState>,
    player_counter: usize,
    player_move_speed: f32,
}

#[derive(Debug, Clone, Copy)]
enum AutoMovePattern {
    Circle,
    LShape,
}

#[derive(Debug, Clone, Copy)]
struct AutoMoveState {
    pattern: AutoMovePattern,
    next_switch_tick: u64,
    pattern_seed: u64,
    shoot_cooldown_ticks: u64,
}

fn state_send_interval_ms() -> u64 {
    env::var("MOBA_STATE_SEND_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(50)
}

fn player_move_speed() -> f32 {
    env::var("MOBA_PLAYER_SPEED")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(400.0)
}

fn auto_move_state(entity_id: EntityId, tick: u64) -> AutoMoveState {
    let pattern_seed = entity_id.0.wrapping_mul(1103515245).wrapping_add(tick);
    AutoMoveState {
        pattern: if pattern_seed & 1 == 0 {
            AutoMovePattern::Circle
        } else {
            AutoMovePattern::LShape
        },
        next_switch_tick: tick + 120 + (pattern_seed % 180),
        pattern_seed,
        shoot_cooldown_ticks: 0,
    }
}

fn projectile_velocity_from_heading(vx: f32, vy: f32) -> (f32, f32) {
    let speed = 600.0;
    let len = (vx * vx + vy * vy).sqrt();
    if len > 0.01 {
        (vx / len * speed, vy / len * speed)
    } else {
        (speed, 0.0)
    }
}

fn auto_move_velocity(pattern: AutoMovePattern, tick: u64, entity_id: EntityId, move_speed: f32) -> (f32, f32) {
    match pattern {
        AutoMovePattern::Circle => {
            let angle = tick as f32 / 18.0 + entity_id.0 as f32 * 0.7;
            (angle.cos() * move_speed, angle.sin() * move_speed)
        }
        AutoMovePattern::LShape => {
            let segment = ((tick / 45) + entity_id.0) % 4;
            match segment {
                0 => (move_speed, 0.0),
                1 => (0.0, move_speed),
                2 => (-move_speed, 0.0),
                _ => (0.0, -move_speed),
            }
        }
    }
}

impl GameServer {
    fn new() -> Self {
        let map = Map::new("MOBA Arena", 1000.0, 1000.0);
        let world = World::new(map);
        let mode: Box<dyn GameMode> = Box::new(MobaMode::new());

        Self {
            world,
            mode,
            client_players: HashMap::new(),
            auto_move_entities: HashMap::new(),
            player_counter: 0,
            player_move_speed: player_move_speed(),
        }
    }

    fn add_player(&mut self, _name: &str) -> (usize, EntityId) {
        let client_id = self.player_counter;
        self.player_counter += 1;

        let spawn_x = if client_id % 2 == 0 { 100.0 } else { 900.0 };
        let spawn_y = if client_id % 2 == 0 { 100.0 } else { 900.0 };

        let player_entity = gameserver_core::Entity::new(
            gen_entity_id(),
            gameserver_core::EntityKind::Player,
            gameserver_core::Transform::new(spawn_x, spawn_y),
            20.0,
        )
        .with_health(gameserver_core::Health::new(100.0));

        let entity_id = player_entity.id;
        self.world.add_entity(player_entity);
        self.client_players.insert(client_id, entity_id);

        println!("👤 Client {} joined (Entity {})", client_id, entity_id.0);
        (client_id, entity_id)
    }

    fn remove_player(&mut self, client_id: usize) {
        if let Some(entity_id) = self.client_players.remove(&client_id) {
            self.world.remove_entity(entity_id);
            self.auto_move_entities.remove(&entity_id);
            println!("👋 Client {} left", client_id);
        }
    }

    fn clear_participants(&mut self) {
        let ids_to_remove: Vec<_> = self
            .world
            .entities
            .iter()
            .filter_map(|(id, entity)| {
                if entity.kind == gameserver_core::EntityKind::Player
                    || entity.kind == gameserver_core::EntityKind::Projectile
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        for id in ids_to_remove {
            self.world.remove_entity(id);
            self.auto_move_entities.remove(&id);
        }

        self.client_players.clear();
        self.player_counter = 0;
        println!("🧹 Cleared all participants");
    }

    fn get_game_state(&self) -> GameState {
        let players = self
            .world
            .entities
            .values()
            .filter(|e| e.kind == gameserver_core::EntityKind::Player)
            .map(|e| PlayerState {
                id: e.id.0,
                x: e.transform.x,
                y: e.transform.y,
                vx: e.physics.vx,
                vy: e.physics.vy,
                health: e.health.map(|h| h.current).unwrap_or(0.0),
                max_health: e.health.map(|h| h.max).unwrap_or(0.0),
            })
            .collect();

        let entities = self
            .world
            .entities
            .values()
            .map(|e| EntityState {
                id: e.id.0,
                x: e.transform.x,
                y: e.transform.y,
                vx: e.physics.vx,
                vy: e.physics.vy,
                kind: match e.kind {
                    gameserver_core::EntityKind::Player => "player".to_string(),
                    gameserver_core::EntityKind::Projectile => "projectile".to_string(),
                    gameserver_core::EntityKind::Ball => "ball".to_string(),
                    gameserver_core::EntityKind::Obstacle => "obstacle".to_string(),
                },
                health: e.health.map(|h| h.current),
                max_health: e.health.map(|h| h.max),
            })
            .collect();

        GameState {
            tick: self.world.tick,
            players,
            entities,
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = "0.0.0.0:8888".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("🎮 MOBA Server started on {}", addr);

    let game_server = Arc::new(Mutex::new(GameServer::new()));
    let game_server_clone = game_server.clone();

    tokio::spawn(async move {
        game_loop(game_server_clone).await;
    });

    loop {
        let (stream, peer_addr) = listener.accept().await.unwrap();
        let game_server_clone = game_server.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, peer_addr, game_server_clone).await {
                eprintln!("Client error: {}", e);
            }
        });
    }
}

async fn game_loop(game_server: Arc<Mutex<GameServer>>) {
    let mut interval = interval(Duration::from_millis((1000.0 / TICK_RATE) as u64));

    loop {
        interval.tick().await;

        {
            let mut server = game_server.lock().await;
            let tick = server.world.tick;
            let move_speed = server.player_move_speed;

            let auto_move_ids: Vec<_> = server.auto_move_entities.keys().copied().collect();
            let mut auto_actions = Vec::with_capacity(auto_move_ids.len());
            let mut auto_velocities = HashMap::with_capacity(auto_move_ids.len());

            for id in auto_move_ids {
                if let Some(state) = server.auto_move_entities.get_mut(&id) {
                    if tick >= state.next_switch_tick {
                        state.pattern_seed = state
                            .pattern_seed
                            .wrapping_mul(1664525)
                            .wrapping_add(1013904223 + id.0);
                        state.pattern = if state.pattern_seed & 1 == 0 {
                            AutoMovePattern::Circle
                        } else {
                            AutoMovePattern::LShape
                        };
                        state.next_switch_tick = tick + 120 + (state.pattern_seed % 180);
                    }

                    let (vx, vy) = auto_move_velocity(state.pattern, tick, id, move_speed);
                    auto_velocities.insert(id, (vx, vy));
                    let shoot = state.shoot_cooldown_ticks == 0;
                    if shoot {
                        state.shoot_cooldown_ticks = 18;
                    } else {
                        state.shoot_cooldown_ticks = state.shoot_cooldown_ticks.saturating_sub(1);
                    }

                    auto_actions.push((id, vx, vy, shoot));
                }
            }

            let mut projectile_despawns = Vec::new();
            let entity_ids: Vec<_> = server.world.entities.keys().copied().collect();
            let map_bounds = (server.world.map.width, server.world.map.height);

            for id in entity_ids {
                if let Some(entity) = server.world.get_entity_mut(id) {
                    if let Some((vx, vy)) = auto_velocities.get(&id) {
                        entity.physics.vx = *vx;
                        entity.physics.vy = *vy;
                    }

                    entity.update_physics(DT);

                    if entity.kind == gameserver_core::EntityKind::Projectile {
                        let out_of_bounds =
                            entity.transform.x < -entity.radius
                                || entity.transform.x > map_bounds.0 + entity.radius
                                || entity.transform.y < -entity.radius
                                || entity.transform.y > map_bounds.1 + entity.radius;
                        if out_of_bounds {
                            projectile_despawns.push(id);
                            continue;
                        }
                    } else {
                        let (clamped_x, clamped_y) = (
                            entity.transform.x.max(entity.radius).min(map_bounds.0 - entity.radius),
                            entity.transform.y.max(entity.radius).min(map_bounds.1 - entity.radius),
                        );
                        entity.transform.x = clamped_x;
                        entity.transform.y = clamped_y;
                    }
                }
            }

            for id in projectile_despawns {
                server.world.remove_entity(id);
            }

            for (id, vx, vy, shoot) in auto_actions {
                if let Some(entity) = server.world.get_entity_mut(id) {
                    entity.physics.vx = vx;
                    entity.physics.vy = vy;
                }
                if shoot {
                    spawn_projectile(&mut server.world, id);
                }
            }

            server.world.step_tick();

            if server.world.tick % 60 == 0 {
                println!("Tick: {}, Players: {}", server.world.tick, server.client_players.len());
            }
        }

        check_collisions(&game_server).await;
    }
}

fn spawn_projectile(world: &mut World, owner_id: EntityId) {
    let (owner_x, owner_y, owner_vx, owner_vy) = if let Some(owner) = world.get_entity(owner_id) {
        (owner.transform.x, owner.transform.y, owner.physics.vx, owner.physics.vy)
    } else {
        return;
    };
    let (proj_vx, proj_vy) = projectile_velocity_from_heading(owner_vx, owner_vy);

    let projectile = gameserver_core::Entity::new(
        gen_entity_id(),
        gameserver_core::EntityKind::Projectile,
        gameserver_core::Transform::new(owner_x, owner_y),
        5.0,
    )
    .with_owner(owner_id);

    let mut projectile = projectile;
    projectile.physics.vx = proj_vx;
    projectile.physics.vy = proj_vy;
    projectile.physics.friction = 0.0;

    world.add_entity(projectile);
}

async fn check_collisions(game_server: &Arc<Mutex<GameServer>>) {
    let mut server = game_server.lock().await;
    let entities: Vec<_> = server
        .world
        .entities
        .values()
        .map(|e| (e.id, e.transform.clone(), e.kind, e.radius, e.owner))
        .collect();

    for (id1, transform1, kind1, radius1, owner1) in &entities {
        for (id2, transform2, kind2, radius2, owner2) in &entities {
            if id1 >= id2 {
                continue;
            }

            let dist = transform1.distance_to(&transform2);
            if dist < radius1 + radius2 {
                if *kind1 == gameserver_core::EntityKind::Projectile && *kind2 == gameserver_core::EntityKind::Player {
                    if *owner1 != Some(*id2) {
                        if let Some(target) = server.world.get_entity_mut(*id2) {
                            if let Some(health) = &mut target.health {
                                health.damage(10.0);
                            }
                        }
                        server.world.remove_entity(*id1);
                    }
                } else if *kind1 == gameserver_core::EntityKind::Player && *kind2 == gameserver_core::EntityKind::Projectile {
                    if *owner2 != Some(*id1) {
                        if let Some(target) = server.world.get_entity_mut(*id1) {
                            if let Some(health) = &mut target.health {
                                health.damage(10.0);
                            }
                        }
                        server.world.remove_entity(*id2);
                    }
                }
            }
        }
    }
}

async fn handle_client(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
    game_server: Arc<Mutex<GameServer>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 Client connected: {}", peer_addr);

    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let mut client_id: Option<usize> = None;
    let mut entity_id: Option<EntityId> = None;

    while let Some(msg_result) = ws_receiver.next().await {
        let msg = msg_result?;

        if let Ok(text) = msg.to_text() {
            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(text) {
                match client_msg {
                    ClientMessage::Join { name } => {
                        let mut server = game_server.lock().await;
                        let (cid, eid) = server.add_player(&name);
                        client_id = Some(cid);
                        entity_id = Some(eid);
                        drop(server);

                        let server = game_server.lock().await;
                        let state = server.get_game_state();
                        let response = serde_json::json!({
                            "type": "joined",
                            "client_id": cid,
                            "entity_id": eid.0,
                            "state": state
                        });

                        ws_sender.send(tokio_tungstenite::tungstenite::Message::text(response.to_string())).await?;
                    }
                    ClientMessage::Input {
                        move_x,
                        move_y,
                        action1,
                    } => {
                        if let Some(eid) = entity_id {
                            let mut server = game_server.lock().await;
                            if server.auto_move_entities.contains_key(&eid) {
                                continue;
                            }
                            let move_speed = server.player_move_speed;
                            if let Some(entity) = server.world.get_entity_mut(eid) {
                                entity.physics.vx = move_x * move_speed;
                                entity.physics.vy = move_y * move_speed;
                                if action1 {
                                    spawn_projectile(&mut server.world, eid);
                                }
                            }
                        }
                    }
                    ClientMessage::Leave => {
                        if let Some(cid) = client_id {
                            let mut server = game_server.lock().await;
                            server.remove_player(cid);
                        }
                        break;
                    }
                    ClientMessage::ClearParticipants => {
                        let mut server = game_server.lock().await;
                        server.clear_participants();

                        let state = server.get_game_state();
                        let broadcast = serde_json::json!({
                            "type": "state",
                            "state": state
                        });

                        ws_sender.send(tokio_tungstenite::tungstenite::Message::text(broadcast.to_string())).await?;
                    }
                    ClientMessage::SetSpeed { speed } => {
                        let mut server = game_server.lock().await;
                        server.player_move_speed = speed.clamp(100.0, 800.0);
                        println!("🏃 Player speed set to {}", server.player_move_speed);
                    }
                    ClientMessage::SetAutoMove { entity_id, enabled } => {
                        let mut server = game_server.lock().await;
                        let eid = EntityId(entity_id);
                        let move_speed = server.player_move_speed;
                        if enabled {
                            let tick = server.world.tick;
                            let state = auto_move_state(eid, tick);
                            server.auto_move_entities.insert(eid, state);
                            if let Some(entity) = server.world.get_entity_mut(eid) {
                                let (vx, vy) = auto_move_velocity(state.pattern, tick, eid, move_speed);
                                entity.physics.vx = vx;
                                entity.physics.vy = vy;
                            }
                        } else {
                            server.auto_move_entities.remove(&eid);
                        }
                    }
                }
            }
        }

        let send_interval = state_send_interval_ms();
        if send_interval > 0 {
            tokio::time::sleep(Duration::from_millis(send_interval)).await;
        }

        let server = game_server.lock().await;
        let state = server.get_game_state();
        let broadcast = serde_json::json!({
            "type": "state",
            "state": state
        });

        ws_sender.send(tokio_tungstenite::tungstenite::Message::text(broadcast.to_string())).await?;
    }

    if let Some(cid) = client_id {
        let mut server = game_server.lock().await;
        server.remove_player(cid);
    }

    println!("📡 Client disconnected: {}", peer_addr);
    Ok(())
}
