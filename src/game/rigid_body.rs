use crate::utils::{collide_continuous, merge_result};
use bevy::{prelude::*, sprite::collide_aabb::Collision};

#[derive(Default, Clone)]
pub struct RigidBody {
    pub velocity: Vec2,
    pub translation: Vec3,

    pub mass: f32,
    pub bounciness: f32,
    pub friction: f32,
    pub kinetic: bool,
}

impl RigidBody {
    pub fn new(mass: f32, bounciness: f32, friction: f32, kinetic: bool) -> Self {
        Self {
            velocity: Vec2::ZERO,
            translation: Vec3::ZERO,
            mass,
            bounciness,
            friction,
            kinetic,
        }
    }
}

pub struct CollisionEvent {
    pub first: Entity,
    pub second: Entity,
    pub collision: Collision,
    pub speed: f32,
    pub time: f32,
}

pub fn rigid_body_added(mut query: Query<(&Transform, &mut RigidBody), Added<RigidBody>>) {
    for (transform, mut rigid_body) in query.iter_mut() {
        rigid_body.translation = transform.translation;
    }
}

pub fn rigid_body_movement(time: Res<Time>, mut query: Query<(&mut RigidBody, &mut Transform)>) {
    for (mut rigid_body, mut transform) in query.iter_mut() {
        rigid_body.translation = transform.translation;

        let velocity = rigid_body.velocity * time.delta_seconds();
        transform.translation += (velocity, 0.0).into();
    }
}

pub fn rigid_body_collision_detection(
    mut event: EventWriter<CollisionEvent>,
    query: Query<(Entity, &Sprite, &Transform, &RigidBody)>,
) {
    for (i, first) in query.iter().enumerate() {
        for second in query
            .iter()
            .skip(i + 1)
            .filter(|second| !first.3.kinetic || !second.3.kinetic)
        {
            if let Some(hit) = collide_continuous(
                first.3.translation,
                first.2.translation,
                first.1.size,
                second.3.translation,
                second.2.translation,
                second.1.size,
            ) {
                event.send(CollisionEvent {
                    first: first.0,
                    second: second.0,
                    collision: hit.collision,
                    speed: (first.3.velocity - second.3.velocity).length(),
                    time: hit.near_time,
                })
            }
        }
    }
}

pub fn rigid_body_collision_resolution(
    mut event: EventReader<CollisionEvent>,
    mut query: Query<(&mut Transform, &mut RigidBody)>,
) {
    for event in event.iter() {
        if let Ok((first, second)) = merge_result(
            query.get_component::<RigidBody>(event.first),
            query.get_component::<RigidBody>(event.second),
        ) {
            let total_mass = first.mass + second.mass;
            let bounciness = first.bounciness * second.bounciness;
            let friction = first.friction * second.friction;
            let velocity = second.velocity - first.velocity;

            let first_kinetic = first.kinetic;
            let second_kinetic = second.kinetic;

            {
                let (mut transform, mut rigid_body) = query.get_mut(event.first).unwrap();

                let mut reflect_x = false;
                let mut reflect_y = false;
                match event.collision {
                    Collision::Left => reflect_x = velocity.x < 0.0,
                    Collision::Right => reflect_x = velocity.x > 0.0,
                    Collision::Top => reflect_y = velocity.y > 0.0,
                    Collision::Bottom => reflect_y = velocity.y < 0.0,
                }

                let mass_factor = if second_kinetic {
                    2.0
                } else {
                    2.0 * (total_mass - rigid_body.mass) / total_mass
                };

                if !rigid_body.kinetic {
                    if reflect_x {
                        rigid_body.velocity.x += bounciness * velocity.x * mass_factor;
                        rigid_body.velocity.y += friction * velocity.y * mass_factor;
                    }

                    if reflect_y {
                        rigid_body.velocity.y += bounciness * velocity.y * mass_factor;
                        rigid_body.velocity.x += friction * velocity.x * mass_factor;
                    }

                    // reset current and previous transform
                    if event.time > 0.0 {
                        transform.translation = rigid_body
                            .translation
                            .lerp(transform.translation, event.time);
                    }
                }
            }

            {
                let (mut transform, mut rigid_body) = query.get_mut(event.second).unwrap();
                let velocity = -velocity;

                let mut reflect_x = false;
                let mut reflect_y = false;
                match event.collision {
                    Collision::Left => reflect_x = velocity.x > 0.0,
                    Collision::Right => reflect_x = velocity.x < 0.0,
                    Collision::Top => reflect_y = velocity.y < 0.0,
                    Collision::Bottom => reflect_y = velocity.y > 0.0,
                }

                let mass_factor = if first_kinetic {
                    2.0
                } else {
                    2.0 * (total_mass - rigid_body.mass) / total_mass
                };

                if !rigid_body.kinetic {
                    if reflect_x {
                        rigid_body.velocity.x += bounciness * velocity.x * mass_factor;
                        rigid_body.velocity.y += friction * velocity.y * mass_factor;
                    }

                    if reflect_y {
                        rigid_body.velocity.y += bounciness * velocity.y * mass_factor;
                        rigid_body.velocity.x += friction * velocity.x * mass_factor;
                    }

                    // reset current and previous transform
                    if event.time > 0.0 {
                        transform.translation = rigid_body
                            .translation
                            .lerp(transform.translation, event.time);
                    }
                }
            }
        }
    }
}

pub struct RigidBodyPlugin;

impl Plugin for RigidBodyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let systems = SystemSet::new()
            .with_system(rigid_body_movement.label("rigid body movement"))
            .with_system(
                rigid_body_collision_resolution
                    .label("rigid body collision resolution")
                    .before("rigid body movement"),
            )
            .with_system(
                rigid_body_collision_detection
                    .label("rigid body collision detection")
                    .before("rigid body collision resolution"),
            )
            .with_system(rigid_body_added.before("rigid body collision detection"));

        app.add_event::<CollisionEvent>()
            .add_stage_after(CoreStage::Update, "physics", SystemStage::parallel())
            .add_system_set_to_stage("physics", systems);
    }
}
