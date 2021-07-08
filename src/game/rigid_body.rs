use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

#[derive(Default, Clone)]
pub struct RigidBody {
    pub mass: f32,
    pub friction: f32,
    pub velocity: Vec2,
}

impl RigidBody {
    pub fn new(mass: f32, friction: f32) -> Self {
        RigidBody {
            mass: mass,
            friction: friction,
            velocity: Vec2::ZERO,
        }
    }
}

pub struct CollisionEvent {
    pub first: Entity,
    pub second: Entity,
    pub collision: Collision,
}

pub fn rigid_body_movement(time: Res<Time>, mut query: Query<(&RigidBody, &mut Transform)>) {
    for (rigid_body, mut transform) in query.iter_mut() {
        let velocity = rigid_body.velocity * time.delta_seconds();
        transform.translation += Vec3::new(velocity.x, velocity.y, 0.0);
    }
}

pub fn rigid_body_collision_detection(
    mut event: EventWriter<CollisionEvent>,
    query: Query<(Entity, &Sprite, &Transform), With<RigidBody>>,
) {
    for (i, (entity, sprite, transform)) in query.iter().enumerate() {
        for (other_entity, other_sprite, other_transform) in query.iter().skip(i + i) {
            if let Some(collision) = collide(
                transform.translation,
                sprite.size,
                other_transform.translation,
                other_sprite.size,
            ) {
                event.send(CollisionEvent {
                    first: entity,
                    second: other_entity,
                    collision: collision,
                });
            }
        }
    }
}

pub fn rigid_body_collision_resolution(
    mut event: EventReader<CollisionEvent>,
    mut query: QuerySet<(Query<&RigidBody>, Query<&mut RigidBody>)>,
) {
    for event in event.iter() {
        if let Ok((first, second)) = query.q0().get(event.first).and_then(|first| {
            query
                .q0()
                .get(event.second)
                .map(|second| (first.clone(), second.clone()))
        }) {
            let total_mass = first.mass + second.mass;

            {
                let mut rigid_body = query.q1_mut().get_mut(event.first).unwrap();
                let velocity = second.velocity - first.velocity;

                let mut reflect_x = false;
                let mut reflect_y = true;
                match event.collision {
                    Collision::Left => reflect_x = velocity.x < 0.0,
                    Collision::Right => reflect_x = velocity.x > 0.0,
                    Collision::Top => reflect_y = velocity.y > 0.0,
                    Collision::Bottom => reflect_y = velocity.y < 0.0,
                }

                let mass_factor = 2.0 * second.mass / total_mass;

                if reflect_x {
                    rigid_body.velocity.x += velocity.x * mass_factor;
                    rigid_body.velocity.y +=
                        (first.friction * second.friction) * velocity.y * mass_factor;
                }

                if reflect_y {
                    rigid_body.velocity.y += velocity.y * mass_factor;
                    rigid_body.velocity.x +=
                        (first.friction * second.friction) * velocity.x * mass_factor;
                }
            }

            {
                let mut rigid_body = query.q1_mut().get_mut(event.second).unwrap();
                let velocity = first.velocity - second.velocity;

                let mut reflect_x = false;
                let mut reflect_y = true;
                match event.collision {
                    Collision::Left => reflect_x = velocity.x > 0.0,
                    Collision::Right => reflect_x = velocity.x < 0.0,
                    Collision::Top => reflect_y = velocity.y < 0.0,
                    Collision::Bottom => reflect_y = velocity.y > 0.0,
                }

                let mass_factor = 2.0 * first.mass / total_mass;

                if reflect_x {
                    rigid_body.velocity.x += velocity.x * mass_factor;
                    rigid_body.velocity.y +=
                        (first.friction * second.friction) * velocity.y * mass_factor;
                }

                if reflect_y {
                    rigid_body.velocity.y += velocity.y * mass_factor;
                    rigid_body.velocity.x +=
                        (first.friction * second.friction) * velocity.x * mass_factor;
                }
            }
        }
    }
}

pub struct RigidBodyPlugin;

impl Plugin for RigidBodyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<CollisionEvent>();
        app.add_system_set(
            SystemSet::new()
                .label("rigid_body")
                .with_system(rigid_body_movement.system().label("rigid_body_movement"))
                .with_system(
                    rigid_body_collision_resolution
                        .system()
                        .label("rigid_body_collision_resolution")
                        .before("rigid_body_movement"),
                )
                .with_system(
                    rigid_body_collision_detection
                        .system()
                        .label("rigid_body_collision_detection")
                        .before("rigid_body_collision_resolution"),
                ),
        );
    }
}
