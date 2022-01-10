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

#[derive(Component)]
pub struct RigidBody {
    pub layer: Layer,
    pub size: Vec2,
    pub inverted_mass: f32,
    pub bounciness: f32,
    pub friction: f32,
}

impl RigidBody {
    pub fn new(layer: Layer, size: Vec2, mass: f32, bounciness: f32, friction: f32) -> Self {
        let inverted_mass = if mass < f32::EPSILON {
            0.0
        } else {
            mass.recip()
        };

        Self {
            layer,
            size,
            inverted_mass,
            bounciness,
            friction,
        }
    }

    pub fn mass(&self) -> f32 {
        self.inverted_mass.recip()
    }
}

#[derive(Default, Component)]
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

pub fn init_motion(mut query: Query<(&Transform, &mut Motion), Added<Motion>>) {
    for (transform, mut motion) in query.iter_mut() {
        motion.translation = transform.translation;
    }
}

pub fn movement(_time: Res<Time>, mut query: Query<(&mut Motion, &mut Transform)>) {
    for (mut motion, mut transform) in query.iter_mut() {
        motion.translation = transform.translation;

        let velocity = motion.velocity * PHYSICS_TIME_STEP as f32;
        transform.translation += velocity.extend(0.0);
    }
}

pub fn collision(
    mut query: Query<(Entity, &RigidBody, &mut Transform, Option<&mut Motion>)>,
    mut events: EventWriter<CollisionEvent>,
) {
    let mut iter = query.iter_combinations_mut();
    while let Some([(e1, rb1, t1, m1), (e2, rb2, t2, m2)]) = iter.fetch_next() {
        if !rb1.layer.test(rb2.layer, Layer::collision_bits) {
            continue;
        }

        let (p1, v1) = match &m1 {
            Some(motion) => (motion.translation, motion.velocity),
            None => (t1.translation, Vec2::ZERO),
        };

        let (p2, v2) = match &m2 {
            Some(motion) => (motion.translation, motion.velocity),
            None => (t2.translation, Vec2::ZERO),
        };
        
        if let Some(hit) =
            collide_continuous(p1, t1.translation, rb1.size, p2, t2.translation, rb2.size)
        {
            let velocity = v2 - v1;

            let bounciness = if rb1.layer.test(rb2.layer, Layer::bounciness_bits) {
                (rb1.bounciness * rb2.bounciness).sqrt()
            } else {
                0.0
            };

            let friction = if rb1.layer.test(rb2.layer, Layer::friction_bits) {
                (rb1.friction * rb2.friction).sqrt()
            } else {
                0.0
            };

            let impulse = (rb1.inverted_mass + rb2.inverted_mass).recip();

            let resolve = |rigid_body: &RigidBody,
                           mut motion: Mut<Motion>,
                           mut transform: Mut<Transform>,
                           velocity: Vec2,
                           normal: Vec2| {
                let normal_speed = velocity.dot(normal);

                // do not process if objects are moving apart
                if normal_speed < 0.0 {
                    return;
                }

                let tangential = (velocity - normal_speed * normal).normalize_or_zero();
                let tangential_speed = velocity.dot(tangential);

                let bounciness = if normal_speed < PHYSICS_REST_SPEED {
                    0.0
                } else {
                    bounciness
                };

                let normal_impulse = (1.0 + bounciness) * impulse * normal_speed;
                let tangential_impulse =
                    (impulse * tangential_speed).min(friction * normal_impulse);

                let normal_delta = normal_impulse * rigid_body.inverted_mass * normal;
                let tangential_delta = tangential_impulse * rigid_body.inverted_mass * tangential;

                motion.velocity += normal_delta + tangential_delta;

                // translation correction
                let depth = hit.depth.abs();
                let near_time = hit.near_time;
                let delta = (motion.translation - transform.translation).truncate();

                if depth > 0.0 {
                    // correct penetration
                    let correction = depth * impulse * rigid_body.inverted_mass * normal;
                    let normal_delta = (delta + correction).dot(normal) * normal;
                    transform.translation += normal_delta.extend(0.0);
                }
                if (near_time + 1.0).abs() < 0.01 {
                    // this is a hack that deals with numerical issue in collision detection
                    let normal_delta = delta.dot(normal) * normal;
                    transform.translation += normal_delta.extend(0.0);
                }
                if near_time > 0.0 {
                    transform.translation =
                        motion.translation.lerp(transform.translation, near_time);
                }
            };

            if let Some(motion) = m1 {
                resolve(rb1, motion, t1, velocity, hit.normal);
            }
            if let Some(motion) = m2 {
                resolve(rb2, motion, t2, -velocity, -hit.normal);
            }

            events.send(CollisionEvent {
                first: e1,
                second: e2,
                velocity,
                impulse,
                bounciness,
                friction,
                hit,
            });
        }
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsSystem {
    Movement,
    Collision,
}

impl SystemLabel for PhysicsSystem {
    fn dyn_clone(&self) -> Box<dyn SystemLabel> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct PhysicsStage;

impl StageLabel for PhysicsStage {
    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        Box::new(self.clone())
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        let systems = SystemSet::new()
            .with_run_criteria(FixedTimestep::step(PHYSICS_TIME_STEP))
            .with_system(init_motion.before(PhysicsSystem::Collision))
            .with_system(movement.label(PhysicsSystem::Movement))
            .with_system(
                collision
                    .label(PhysicsSystem::Collision)
                    .after(PhysicsSystem::Movement),
            );

        app.add_event::<CollisionEvent>()
            .add_stage_after(CoreStage::Update, PhysicsStage, SystemStage::parallel())
            .add_system_set_to_stage(PhysicsStage, systems);
    }
}
