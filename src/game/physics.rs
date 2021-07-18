use crate::{
    config::{PHYSICS_REST_SPEED, PHYSICS_TIME_STEP},
    utils::*,
};
use bevy::{core::FixedTimestep, prelude::*};

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

pub struct RigidBody {
    pub layer: Layer,
    pub inverted_mass: f32,
    pub bounciness: f32,
    pub friction: f32,
}

impl RigidBody {
    pub fn new(layer: Layer, mass: f32, bounciness: f32, friction: f32) -> Self {
        let inverted_mass = if mass < f32::EPSILON {
            0.0
        } else {
            mass.recip()
        };

        Self {
            layer,
            inverted_mass,
            bounciness,
            friction,
        }
    }

    pub fn mass(&self) -> f32 {
        self.inverted_mass.recip()
    }
}

#[derive(Default)]
pub struct Motion {
    pub velocity: Vec2,
    pub translation: Vec3,
}

pub struct CollisionEvent {
    pub first: Entity,
    pub second: Entity,
    pub velocity: Vec2,
    pub impulse: f32,
    pub bounciness: f32,
    pub friction: f32,
    pub hit: Hit,
}

pub fn motion_added(mut query: Query<(&Transform, &mut Motion), Added<Motion>>) {
    for (transform, mut motion) in query.iter_mut() {
        motion.translation = transform.translation;
    }
}

pub fn movement(_time: Res<Time>, mut query: Query<(&mut Motion, &mut Transform)>) {
    for (mut motion, mut transform) in query.iter_mut() {
        motion.translation = transform.translation;

        let velocity = motion.velocity * PHYSICS_TIME_STEP as f32;
        transform.translation += (velocity, 0.0).into();
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
                let velocity = {
                    let first = first.4.map_or(Vec2::ZERO, |motion| motion.velocity);
                    let second = second.4.map_or(Vec2::ZERO, |motion| motion.velocity);
                    second - first
                };

                let bounciness = if first.3.layer.test(second.3.layer, Layer::bounciness_bits) {
                    (first.3.bounciness * second.3.bounciness).sqrt()
                } else {
                    0.0
                };

                let friction = if first.3.layer.test(second.3.layer, Layer::friction_bits) {
                    (first.3.friction * second.3.friction).sqrt()
                } else {
                    0.0
                };

                events.send(CollisionEvent {
                    first: first.0,
                    second: second.0,
                    impulse: (first.3.inverted_mass + second.3.inverted_mass).recip(),
                    velocity,
                    bounciness,
                    friction,
                    hit,
                })
            }
        }
    }
}

pub fn collision_resolution(
    mut events: EventReader<CollisionEvent>,
    mut query: Query<(&RigidBody, &mut Motion)>,
) {
    for event in events.iter() {
        let mut closure = |entity: Entity, velocity: Vec2, normal: Vec2| {
            if let Ok((rigid_body, mut motion)) = query.get_mut(entity) {
                let normal_speed = velocity.dot(normal);

                // do not process if objects are moving apart
                if normal_speed < 0.0 {
                    return;
                }

                let normal_velocity = normal_speed * normal;
                let tangential_velocity = velocity - normal_velocity;

                let bounciness = if normal_speed < PHYSICS_REST_SPEED {
                    0.0
                } else {
                    event.bounciness
                };

                let normal_velocity =
                    (1.0 + bounciness) * event.impulse * rigid_body.inverted_mass * normal_velocity;
                let tangential_velocity =
                    event.friction * event.impulse * rigid_body.inverted_mass * tangential_velocity;

                motion.velocity += normal_velocity + tangential_velocity;
            }
        };

        closure(event.first, event.velocity, event.hit.normal);
        closure(event.second, -event.velocity, -event.hit.normal);
    }
}

pub fn translation_correction(
    mut events: EventReader<CollisionEvent>,
    mut query: Query<(&RigidBody, &Motion, &mut Transform)>,
) {
    for event in events.iter() {
        let mut closure = |entity: Entity, normal: Vec2| {
            if let Ok((rigid_body, motion, mut transform)) = query.get_mut(entity) {
                let depth = event.hit.depth.abs();
                let delta = (motion.translation - transform.translation).truncate();

                if depth > 0.0 {
                    // correct penetration
                    let correction = depth * event.impulse * rigid_body.inverted_mass * normal;
                    let normal_delta = (delta + correction).dot(normal) * normal;
                    transform.translation += normal_delta.extend(0.0);
                }
                if (event.hit.near_time + 1.0).abs() < 0.01 {
                    // this is a hack that deals with numerical issue in collision detection
                    let normal_delta = delta.dot(normal) * normal;
                    transform.translation += normal_delta.extend(0.0);
                }
                if event.hit.near_time > 0.0 {
                    transform.translation = motion
                        .translation
                        .lerp(transform.translation, event.hit.near_time);
                }
            }
        };

        closure(event.first, event.hit.normal);
        closure(event.second, -event.hit.normal);
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
            .with_run_criteria(FixedTimestep::step(PHYSICS_TIME_STEP))
            .with_system(translation_correction.label(LABEL_TRANSLATION_CORRECTION))
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
