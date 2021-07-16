use crate::{game::prelude::*, utils::Damp};
use bevy::prelude::*;

#[derive(new)]
pub struct Enemy {
    max_speed: f32,
    damp: f32,
}

#[derive(Default)]
pub struct EnemyController {
    velocity: Vec2,
}

pub fn enemy_movement(
    time: Res<Time>,
    mut query: Query<(&Enemy, Option<&EnemyController>, &mut Motion)>,
) {
    for (enemy, controller, mut motion) in query.iter_mut() {
        let velocity = controller
            .map(|controller| controller.velocity)
            .unwrap_or_default();
        motion.velocity = motion
            .velocity
            .damp(velocity, enemy.damp, time.delta_seconds())
            .clamp_length_max(enemy.max_speed);
    }
}
