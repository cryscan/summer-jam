use bevy::prelude::*;

#[derive(Default)]
pub struct RigidBody {
    pub mass: f32,
    pub velocity: Vec2,
}

impl RigidBody {
    pub fn new(mass: f32) -> Self {
        RigidBody {
            mass: mass,
            velocity: Vec2::ZERO,
        }
    }
}

pub fn rigid_body_movement(time: Res<Time>, mut query: Query<(&RigidBody, &mut Transform)>) {
    for (rigid_body, mut transform) in query.iter_mut() {
        let velocity = rigid_body.velocity * time.delta_seconds();
        transform.translation += Vec3::new(velocity.x, velocity.y, 0.0);
    }
}
