use super::rigid_body::Motion;
use bevy::prelude::*;

pub struct Ball {
    pub gravity: f32,
    pub timer: Timer,
}

pub fn ball_movement(time: Res<Time>, mut query: Query<(&Ball, &mut Motion)>) {
    for (ball, mut motion) in query.iter_mut() {
        motion.velocity.y += ball.gravity * time.delta_seconds();
    }
}
