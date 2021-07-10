use super::rigid_body::RigidBody;
use bevy::prelude::*;

pub struct Ball {
    pub gravity: f32,
}

pub fn ball_movement(time: Res<Time>, mut query: Query<(&Ball, &mut RigidBody)>) {
    for (ball, mut rigid_body) in query.iter_mut() {
        rigid_body.velocity.y += ball.gravity * time.delta_seconds();
    }
}
