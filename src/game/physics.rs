use crate::{config::REST_SPEED, utils::*};
use bevy::{prelude::*, sprite::collide_aabb::Collision};
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Layer {
    Boundary = 0,
    Separate = 1,
    Ball = 2,
    Player = 3,
}

impl Layer {
    pub const NONE: u8 = u8::MIN;
    pub const ALL: u8 = u8::MAX;

    const BOUNDARY: u8 = Self::Boundary.bits();
    const BALL: u8 = Self::Ball.bits();
    const PLAYER: u8 = Self::Player.bits();

    pub const fn bits(self) -> u8 {
        1 << self as u8
    }

    pub const fn collision_bits(self) -> u8 {
        match self {
            Self::Boundary => Self::BALL | Self::PLAYER,
            Self::Separate => Self::PLAYER,
            Self::Ball => Self::BOUNDARY | Self::BALL | Self::PLAYER,
            Self::Player => Self::ALL,
        }
    }

    pub const fn bounciness_bits(self) -> u8 {
        match self {
            Self::Boundary => Self::BALL,
            Self::Separate => Self::NONE,
            Self::Ball => Self::ALL,
            Self::Player => Self::BALL | Self::PLAYER,
        }
    }

    pub const fn friction_bits(self) -> u8 {
        match self {
            Self::Boundary => Self::BALL,
            Self::Separate => Self::NONE,
            Self::Ball => Self::ALL,
            Self::Player => Self::BALL | Self::PLAYER,
        }
    }

    pub fn test(self, other: Self, method: fn(Self) -> u8) -> bool {
        (method(self) & other.bits()) != Self::NONE
    }
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
    mut events: EventReader<CollisionEvent>,
    mut query: Query<(&Motion, &mut Transform)>,
) {
    for event in events.iter() {
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
    mut events: EventWriter<CollisionEvent>,
    query: Query<(Entity, &Sprite, &Transform, &RigidBody, Option<&Motion>)>,
) {
    for (i, first) in query.iter().enumerate() {
        for second in query
            .iter()
            .skip(i + 1)
            .filter(|second| first.3.layer.test(second.3.layer, Layer::collision_bits))
        {
            if let Some(hit) = collide_continuous(
                first
                    .4
                    .map_or(first.2.translation, |motion| motion.translation),
                first.2.translation,
                first.1.size,
                second
                    .4
                    .map_or(second.2.translation, |motion| motion.translation),
                second.2.translation,
                second.1.size,
            ) {
                events.send(CollisionEvent {
                    first: first.0,
                    second: second.0,
                    hit,
                })
            }
        }
    }
}

pub fn collision_resolution(
    mut events: EventReader<CollisionEvent>,
    mut query: QuerySet<(
        Query<(&RigidBody, Option<&Motion>)>,
        Query<Option<(&mut Motion, &mut Transform)>>,
    )>,
) {
    for event in events.iter() {
        let mut closure = || -> Result<(), Box<dyn Error>> {
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
            let bounciness = if first.layer.test(second.layer, Layer::bounciness_bits) {
                (first.bounciness * second.bounciness).sqrt()
            } else {
                0.0
            };
            let friction = if first.layer.test(second.layer, Layer::friction_bits) {
                (first.friction * second.friction).sqrt()
            } else {
                0.0
            };

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
                let correct_translation = |translation: &mut f32, previous: f32| {
                    if event.hit.depth.abs() > 0.0 {
                        *translation = previous + mass_factor * event.hit.depth;
                    } else if (event.hit.near_time + 1.0).abs() < 0.01 {
                        *translation = previous;
                    }
                };

                if reflect_x {
                    motion.velocity.x += bounce_factor(velocity.x) * mass_factor * velocity.x;
                    motion.velocity.y += friction * velocity.y;
                    correct_translation(&mut transform.translation.x, motion.translation.x);
                }

                if reflect_y {
                    motion.velocity.y += bounce_factor(velocity.y) * mass_factor * velocity.y;
                    motion.velocity.x += friction * velocity.x;
                    correct_translation(&mut transform.translation.y, motion.translation.y);
                }
            }

            if let Some((mut motion, mut transform)) = query.q1_mut().get_mut(event.second)? {
                let velocity = -velocity;
                let mass_factor = if kinetic.0 { 1.0 } else { 1.0 - mass_factor };
                let correct_translation = |translation: &mut f32, previous: f32| {
                    if event.hit.depth.abs() > 0.0 {
                        *translation = previous - mass_factor * event.hit.depth;
                    } else if (event.hit.near_time + 1.0).abs() < 0.01 {
                        *translation = previous;
                    }
                };

                if reflect_x {
                    motion.velocity.x += bounce_factor(velocity.x) * mass_factor * velocity.x;
                    motion.velocity.y += friction * velocity.y;
                    correct_translation(&mut transform.translation.x, motion.translation.x);
                }

                if reflect_y {
                    motion.velocity.y += bounce_factor(velocity.y) * mass_factor * velocity.y;
                    motion.velocity.x += friction * velocity.x;
                    correct_translation(&mut transform.translation.y, motion.translation.y);
                }
            }

            Ok(())
        };

        closure().unwrap_or_default()
    }
}

const LABEL_TRANSLATION_CORRECTION: &str = "translation correction";
const LABEL_MOVEMENT: &str = "movement";
const LABEL_COLLISION_RESOLUTION: &str = "collision resolution";
const LABEL_COLLISION_DETECTION: &str = "collision detection";

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
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
