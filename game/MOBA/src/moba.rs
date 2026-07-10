use gameserver_core::{GameMode, World, PlayerInput, Entity, EntityKind, Transform, Health, gen_entity_id};

pub struct MobaMode;

impl MobaMode {
    pub fn new() -> Self {
        Self
    }
}

impl GameMode for MobaMode {
    fn init(&mut self, _world: &mut World) {}

    fn handle_input(&mut self, world: &mut World, input: PlayerInput) {
        if let Some(entity) = world.get_entity_mut(input.player_id) {
            let speed = 250.0;
            entity.physics.vx = input.move_x * speed;
            entity.physics.vy = input.move_y * speed;

            if input.action1 {
                spawn_projectile(world, input.player_id);
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

        check_collisions(world);
    }

    fn name(&self) -> &'static str {
        "MOBAMode"
    }
}

fn spawn_projectile(world: &mut World, owner_id: gameserver_core::EntityId) {
    let (owner_x, owner_y) = if let Some(owner) = world.get_entity(owner_id) {
        (owner.transform.x, owner.transform.y)
    } else {
        return;
    };

    let projectile = Entity::new(
        gen_entity_id(),
        EntityKind::Projectile,
        Transform::new(owner_x, owner_y),
        5.0,
    )
    .with_owner(owner_id);

    world.add_entity(projectile);
}

fn check_collisions(world: &mut World) {
    let entities: Vec<_> = world
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
                if *kind1 == EntityKind::Projectile && *kind2 == EntityKind::Player {
                    if *owner1 != Some(*id2) {
                        if let Some(target) = world.get_entity_mut(*id2) {
                            if let Some(health) = &mut target.health {
                                health.damage(10.0);
                            }
                        }
                        world.remove_entity(*id1);
                    }
                } else if *kind1 == EntityKind::Player && *kind2 == EntityKind::Projectile {
                    if *owner2 != Some(*id1) {
                        if let Some(target) = world.get_entity_mut(*id1) {
                            if let Some(health) = &mut target.health {
                                health.damage(10.0);
                            }
                        }
                        world.remove_entity(*id2);
                    }
                }
            }
        }
    }
}
