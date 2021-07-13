use crate::{config::REST_SPEED, utils::*};
use bevy::{prelude::*, sprite::collide_aabb::Collision};
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Boundary,
    Ball,
    Player,
}

#[derive(new)]
pub struct RigidBody {
    pub layer: Layer,
    pub mass: f32,
    pub bounciness: f32,
    pub friction: f32,
}

#[derive(Default)]
pub struct Motion {
    pub velocity: Vec2,
    pub translation: Vec3,
}

impl Layer {
    fn bounciness_multiplier(self, other: Self) -> f32 {
        match (self, other) {
            (Layer::Boundary, Layer::Player) => 0.0,
            (Layer::Player, Layer::Boundary) => 0.0,
            _ => 1.0,
        }
    }

    fn friction_multiplier(self, other: Self) -> f32 {
        match (self, other) {
            (Layer::Boundary, Layer::Player) => 0.0,
            (Layer::Player, Layer::Boundary) => 0.0,
            _ => 1.0,
        }
    }
}

pub struct CollisionEvent {
    pub first: Entity,
    pub second: Entity,
    pub hit: Hit,
}

pub fn motion_added(mut query: Query<(&Transform, &mut Motion), Added<Motion>>) {
    for (transform, mut motion) in query.iter_mut() {
        motion.translation = transform.translation;
    }
}

pub fn movement(time: Res<Time>, mut query: Query<(&mut Motion, &mut Transform)>) {
    for (mut motion, mut transform) in query.iter_mut() {
        motion.translation = transform.translation;

        let velocity = motion.velocity * time.delta_seconds();
        transform.translation += (velocity, 0.0).into();
    }
}

pub fn continuous_translation_correction(
    mut event: EventReader<CollisionEvent>,
    mut query: Query<(&Motion, &mut Transform)>,
) {
    for event in event.iter() {
        if let Ok((motion, mut transform)) = query.get_mut(event.first) {
            if event.hit.near_time > 0.0 {
                transform.translation = motion
                    .translation
                    .lerp(transform.translation, event.hit.near_time);
            }
        }

        if let Ok((motion, mut transform)) = query.get_mut(event.second) {
            if event.hit.near_time > 0.0 {
                transform.translation = motion
                    .translation
                    .lerp(transform.translation, event.hit.near_time);
            }
        }
    }
}

pub fn collision_detection(
    mut event: EventWriter<CollisionEvent>,
    query: Query<(Entity, &Sprite, &Transform, &RigidBody, Option<&Motion>)>,
) {
    for (i, first) in query.iter().enumerate() {
        for second in query.iter().skip(i + 1) {
            let (a_prev_pos, b_prev_pos) = match (first.4, second.4) {
                (None, None) => continue,
                (None, Some(motion)) => (first.2.translation, motion.translation),
                (Some(motion), None) => (motion.translation, second.2.translation),
                (Some(first), Some(second)) => (first.translation, second.translation),
            };

            if let Some(hit) = collide_continuous(
                a_prev_pos,
                first.2.translation,
                first.1.size,
                b_prev_pos,
                second.2.translation,
                second.1.size,
            ) {
                event.send(CollisionEvent {
                    first: first.0,
                    second: second.0,
                    hit,
                })
            }
        }
    }
}

pub fn collision_resolution(
    mut event: EventReader<CollisionEvent>,
    mut query: QuerySet<(
        Query<(&RigidBody, Option<&Motion>)>,
        Query<Option<(&mut Motion, &mut Transform)>>,
    )>,
) {
    for event in event.iter() {
        let mut resolve = || -> Result<(), Box<dyn Error>> {
            let first = query.q0().get(event.first)?;
            let second = query.q0().get(event.second)?;
            let kinetic = (first.1.is_none(), second.1.is_none());

            let velocity = {
                let first = first.1.map_or(Vec2::ZERO, |motion| motion.velocity);
                let second = second.1.map_or(Vec2::ZERO, |motion| motion.velocity);
                second - first
            };

            let mut reflect_x = false;
            let mut reflect_y = false;
            match event.hit.collision {
                Collision::Left => reflect_x = velocity.x < 0.0,
                Collision::Right => reflect_x = velocity.x > 0.0,
                Collision::Top => reflect_y = velocity.y > 0.0,
                Collision::Bottom => reflect_y = velocity.y < 0.0,
            }

            let (first, second) = (first.0, second.0);
            let bounciness = first.layer.bounciness_multiplier(second.layer)
                * first.bounciness
                * second.bounciness;
            let friction =
                first.layer.friction_multiplier(second.layer) * first.friction * second.friction;

            let mass_factor = second.mass / (first.mass + second.mass);
            let bounce_factor = |velocity: f32| {
                if velocity.abs() < REST_SPEED {
                    1.0
                } else {
                    1.0 + bounciness
                }
            };

            if let Some((mut motion, mut transform)) = query.q1_mut().get_mut(event.first)? {
                let mass_factor = if kinetic.1 { 1.0 } else { mass_factor };

                if reflect_x {
                    motion.velocity.x += bounce_factor(velocity.x) * mass_factor * velocity.x;
                    motion.velocity.y += friction * velocity.y;

                    if event.hit.depth.abs() > 0.0 {
                        transform.translation.x =
                            motion.translation.x + mass_factor * event.hit.depth;
                    } else if event.hit.near_time < f32::EPSILON - 1.0 {
                        transform.translation.x = motion.translation.x;
                    }
                }

                if reflect_y {
                    motion.velocity.y += bounce_factor(velocity.y) * mass_factor * velocity.y;
                    motion.velocity.x += friction * velocity.x;

                    if event.hit.depth.abs() > 0.0 {
                        transform.translation.y =
                            motion.translation.y + mass_factor * event.hit.depth;
                    } else if event.hit.near_time < f32::EPSILON - 1.0 {
                        transform.translation.y = motion.translation.y;
                    }
                }
            }

            if let Some((mut motion, mut transform)) = query.q1_mut().get_mut(event.second)? {
                let velocity = -velocity;
                let mass_factor = if kinetic.0 { 1.0 } else { 1.0 - mass_factor };

                if reflect_x {
                    motion.velocity.x += bounce_factor(velocity.x) * mass_factor * velocity.x;
                    motion.velocity.y += friction * velocity.y;

                    if event.hit.depth.abs() > 0.0 {
                        transform.translation.x =
                            motion.translation.x - mass_factor * event.hit.depth;
                    } else if event.hit.near_time < f32::EPSILON - 1.0 {
                        transform.translation.x = motion.translation.x;
                    }
                }

                if reflect_y {
                    motion.velocity.y += bounce_factor(velocity.y) * mass_factor * velocity.y;
                    motion.velocity.x += friction * velocity.x;

                    if event.hit.depth.abs() > 0.0 {
                        transform.translation.y =
                            motion.translation.y - mass_factor * event.hit.depth;
                    } else if event.hit.near_time < f32::EPSILON - 1.0 {
                        transform.translation.y = motion.translation.y;
                    }
                }
            }

            Ok(())
        };

        resolve().unwrap_or_default();
    }
}

const LABEL_TRANSLATION_CORRECTION: &str = "translation correction";
const LABEL_MOVEMENT: &str = "movement";
const LABEL_COLLISION_RESOLUTION: &str = "collision resolution";
const LABEL_COLLISION_DETECTION: &str = "collision detection";

pub struct RigidBodyPlugin;

impl Plugin for RigidBodyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let systems = SystemSet::new()
            .with_system(continuous_translation_correction.label(LABEL_TRANSLATION_CORRECTION))
            .with_system(
                collision_resolution
                    .label(LABEL_COLLISION_RESOLUTION)
                    .before(LABEL_TRANSLATION_CORRECTION),
            )
            .with_system(
                collision_detection
                    .label(LABEL_COLLISION_DETECTION)
                    .before(LABEL_COLLISION_RESOLUTION),
            )
            .with_system(
                movement
                    .label(LABEL_MOVEMENT)
                    .before(LABEL_COLLISION_DETECTION),
            )
            .with_system(motion_added.before(LABEL_COLLISION_DETECTION));

        app.add_event::<CollisionEvent>()
            .add_stage_after(CoreStage::Update, "physics", SystemStage::parallel())
            .add_system_set_to_stage("physics", systems);
    }
}
