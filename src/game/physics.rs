use crate::{
    constants::{PHYSICS_REST_SPEED, PHYSICS_TIME_STEP},
    utils::*,
    TimeScale,
};
use bevy::{prelude::*, render::view::RenderLayers, time::FixedTimestep};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        let systems = SystemSet::new()
            .with_run_criteria(FixedTimestep::step(PHYSICS_TIME_STEP as f64))
            .with_system(init_motion)
            .with_system(movement)
            .with_system(collision.after(init_motion).after(movement));

        app.add_event::<CollisionEvent>()
            .add_system_set_to_stage(CoreStage::PostUpdate, systems);
    }
}

#[derive(Clone, Component)]
pub struct PhysicsLayers {
    pub collision: RenderLayers,
    pub bounciness: RenderLayers,
    pub friction: RenderLayers,
}

impl PhysicsLayers {
    pub const BALL: Self = Self {
        collision: RenderLayers::layer(1).with(2),
        bounciness: RenderLayers::layer(1).with(2),
        friction: RenderLayers::layer(1).with(2),
    };
    pub const PLAYER: Self = Self {
        collision: RenderLayers::layer(1).with(3).with(4),
        bounciness: RenderLayers::layer(1),
        friction: RenderLayers::layer(1),
    };
    pub const BOUNDARY: Self = Self {
        collision: RenderLayers::layer(2).with(3),
        bounciness: RenderLayers::layer(2),
        friction: RenderLayers::layer(2),
    };
    pub const SEPARATE: Self = Self {
        collision: RenderLayers::layer(4),
        bounciness: RenderLayers::none(),
        friction: RenderLayers::none(),
    };
}

#[derive(Component)]
pub struct RigidBody {
    pub size: Vec2,
    pub inverted_mass: f32,
    pub bounciness: f32,
    pub friction: f32,
}

impl RigidBody {
    pub fn new(size: Vec2, mass: f32, bounciness: f32, friction: f32) -> Self {
        let inverted_mass = if mass < f32::EPSILON {
            0.0
        } else {
            mass.recip()
        };

        Self {
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
    pub entities: [Entity; 2],
    pub delta_velocities: [Vec2; 2],
    pub bounciness: f32,
    pub friction: f32,
    pub hit: Hit,
}

fn init_motion(mut query: Query<(&Transform, &mut Motion), Added<Motion>>) {
    for (transform, mut motion) in query.iter_mut() {
        motion.translation = transform.translation;
    }
}

fn movement(time_scale: Res<TimeScale>, mut query: Query<(&mut Motion, &mut Transform)>) {
    for (mut motion, mut transform) in query.iter_mut() {
        motion.translation = transform.translation;

        let delta_time = PHYSICS_TIME_STEP * time_scale.0;
        transform.translation += motion.velocity.extend(0.0) * delta_time;
    }
}

#[allow(clippy::type_complexity)]
fn collision(
    time_scale: Res<TimeScale>,
    mut query: Query<(
        Entity,
        &RigidBody,
        &mut Transform,
        Option<&mut Motion>,
        &PhysicsLayers,
    )>,
    mut events: EventWriter<CollisionEvent>,
) {
    let delta_time = PHYSICS_TIME_STEP * time_scale.0;
    let mut combinations = query.iter_combinations_mut();
    while let Some([(e1, rb1, t1, m1, pl1), (e2, rb2, t2, m2, pl2)]) = combinations.fetch_next() {
        if !pl1.collision.intersects(&pl2.collision) {
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

        if let Some(hit) = collide(
            &Collider {
                previous_position: p1.truncate(),
                position: t1.translation.truncate(),
                size: rb1.size,
            },
            &Collider {
                previous_position: p2.truncate(),
                position: t2.translation.truncate(),
                size: rb2.size,
            },
        ) {
            let bounciness = if pl1.bounciness.intersects(&pl2.bounciness) {
                (rb1.bounciness * rb2.bounciness).sqrt()
            } else {
                0.0
            };
            let friction = if pl1.friction.intersects(&pl2.friction) {
                (rb1.friction * rb2.friction).sqrt()
            } else {
                0.0
            };
            let impulse = (rb1.inverted_mass + rb2.inverted_mass).recip();
            let normal = hit.normal();

            let resolve = |rigid_body: &RigidBody,
                           mut motion: Mut<Motion>,
                           mut transform: Mut<Transform>,
                           velocity: Vec2,
                           normal: Vec2|
             -> Option<Vec2> {
                let normal_speed = velocity.dot(normal);

                // do not process if objects are moving apart
                if normal_speed < 0.0 {
                    return None;
                }

                let tan = (velocity - normal_speed * normal).normalize_or_zero();
                let tan_speed = velocity.dot(tan);

                let bounciness = if normal_speed < PHYSICS_REST_SPEED {
                    0.0
                } else {
                    bounciness
                };

                let normal_impulse = (1.0 + bounciness) * impulse * normal_speed;
                let tan_impulse = (impulse * tan_speed).min(friction * normal_impulse);

                let normal_delta = normal_impulse * rigid_body.inverted_mass;
                let tan_delta = tan_impulse * rigid_body.inverted_mass;
                let delta_velocity = normal_delta * normal + tan_delta * tan;
                motion.velocity += delta_velocity;

                // translation correction
                match &hit {
                    Hit::Penetration(x) => {
                        let delta = (motion.translation - transform.translation)
                            .truncate()
                            .dot(normal);
                        let depth = x.depth.abs();

                        // compensate penetration based on masses
                        let correction = depth * impulse * rigid_body.inverted_mass;
                        // compensate normal impulse to be applied in the next physics update
                        let debounce = (1.0 - bounciness) * normal_delta * delta_time;
                        let normal_delta = delta + correction - debounce;
                        transform.translation += normal_delta * normal.extend(0.0);
                    }
                    Hit::Cast(x) => {
                        if x.near_time > 0.0 {
                            transform.translation =
                                motion.translation.lerp(transform.translation, x.near_time);
                        }
                    }
                };

                Some(delta_velocity)
            };

            let mut delta_velocities: [Vec2; 2] = Default::default();
            if let Some(motion) = m1 {
                delta_velocities[0] = resolve(rb1, motion, t1, v2 - v1, normal).unwrap_or_default();
            }
            if let Some(motion) = m2 {
                delta_velocities[1] =
                    resolve(rb2, motion, t2, v1 - v2, -normal).unwrap_or_default();
            }

            let mut entities = [e1, e2];
            entities.sort();
            events.send(CollisionEvent {
                entities,
                delta_velocities,
                bounciness,
                friction,
                hit,
            });
        }
    }
}
