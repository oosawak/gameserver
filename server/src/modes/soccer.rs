use gameserver_core::{GameMode, World, PlayerInput, Entity, EntityKind, Transform, Health, gen_entity_id, EntityId};

pub struct SoccerMode {
    ball_id: Option<EntityId>,
}

impl SoccerMode {
    pub fn new() -> Self {
        Self { ball_id: None }
    }
}

impl GameMode for SoccerMode {
    fn init(&mut self, world: &mut World) {
        // スポーン配置
        let player1 = Entity::new(gen_entity_id(), EntityKind::Player, Transform::new(200.0, 500.0), 20.0)
            .with_health(Health::new(100.0));
        world.add_entity(player1);

        let player2 = Entity::new(gen_entity_id(), EntityKind::Player, Transform::new(800.0, 500.0), 20.0)
            .with_health(Health::new(100.0));
        world.add_entity(player2);

        // ボール生成
        let ball = Entity::new(gen_entity_id(), EntityKind::Ball, Transform::new(500.0, 500.0), 10.0);
        self.ball_id = Some(world.add_entity(ball));

        println!("⚽ Soccer Mode initialized!");
    }

    fn handle_input(&mut self, world: &mut World, input: PlayerInput) {
        if let Some(entity) = world.get_entity_mut(input.player_id) {
            let speed = 200.0;
            entity.physics.vx = input.move_x * speed;
            entity.physics.vy = input.move_y * speed;

            if input.action1 {
                if let Some(ball_id) = self.ball_id {
                    try_kick_ball(world, input.player_id, ball_id);
                }
            }
        }
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entity_ids: Vec<_> = world.entities.keys().copied().collect();
        let map_bounds = (world.map.width, world.map.height);

        for id in entity_ids {
            if let Some(entity) = world.get_entity_mut(id) {
                entity.update_physics(dt);

                let (clamped_x, clamped_y) = (
                    entity.transform.x.max(entity.radius).min(map_bounds.0 - entity.radius),
                    entity.transform.y.max(entity.radius).min(map_bounds.1 - entity.radius),
                );
                entity.transform.x = clamped_x;
                entity.transform.y = clamped_y;
            }
        }

        if let Some(ball_id) = self.ball_id {
            check_goal(world, ball_id);
        }
    }

    fn name(&self) -> &'static str {
        "SoccerMode"
    }
}

fn try_kick_ball(world: &mut World, player_id: EntityId, ball_id: EntityId) {
    let (player_x, player_y, player_radius) = if let Some(player) = world.get_entity(player_id) {
        (player.transform.x, player.transform.y, player.radius)
    } else {
        return;
    };

    if let Some(ball) = world.get_entity(ball_id) {
        let dist = ((player_x - ball.transform.x).powi(2) + (player_y - ball.transform.y).powi(2)).sqrt();
        if dist < player_radius + ball.radius + 30.0 {
            if let Some(ball) = world.get_entity_mut(ball_id) {
                let dx = ball.transform.x - player_x;
                let dy = ball.transform.y - player_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len > 0.0 {
                    ball.physics.vx = (dx / len) * 400.0;
                    ball.physics.vy = (dy / len) * 400.0;
                }
            }
        }
    }
}

fn check_goal(world: &mut World, ball_id: EntityId) {
    if let Some(ball) = world.get_entity(ball_id) {
        if ball.transform.x < 50.0 {
            println!("🎉 GOAL! Team 2 scored!");
            if let Some(ball) = world.get_entity_mut(ball_id) {
                ball.transform.x = 500.0;
                ball.transform.y = 500.0;
                ball.physics.vx = 0.0;
                ball.physics.vy = 0.0;
            }
        } else if ball.transform.x > 950.0 {
            println!("🎉 GOAL! Team 1 scored!");
            if let Some(ball) = world.get_entity_mut(ball_id) {
                ball.transform.x = 500.0;
                ball.transform.y = 500.0;
                ball.physics.vx = 0.0;
                ball.physics.vy = 0.0;
            }
        }
    }
}
