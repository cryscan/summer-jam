use super::rigid_body::Motion;
use bevy::prelude::*;

pub struct Ball {
    pub gravity: f32,
    pub timer: Timer,
}

pub fn ball_setup(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Ball), Without<Motion>>,
) {
    for (entity, mut ball) in query.iter_mut() {
        if ball.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).insert(Motion::default());
        }
    }
}

pub fn ball_movement(time: Res<Time>, mut query: Query<(&Ball, &mut Motion)>) {
    for (ball, mut motion) in query.iter_mut() {
        motion.velocity.y += ball.gravity * time.delta_seconds();
    }
}
