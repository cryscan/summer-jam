use crate::utils::Damp;
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::ops::Add;

pub struct Velocity(pub Vec2);

pub struct Player {
    pub speed: f32,
    pub damp: f32,
}

pub fn player_movement(
    time: Res<Time>,
    mut mouse_motion_event: EventReader<MouseMotion>,
    mut query: Query<(&Player, &mut Velocity, &mut Transform)>,
) {
    if let Ok((player, mut velocity, mut transform)) = query.single_mut() {
        let delta: Vec2 = mouse_motion_event
            .iter()
            .map(|mouse_motion| mouse_motion.delta)
            .fold(Vec2::ZERO, Vec2::add);

        velocity.0 = velocity
            .0
            .damp(delta * player.speed, player.damp, time.delta_seconds());
        let displacement = velocity.0 * time.delta_seconds();
        transform.translation += Vec3::new(displacement.x, -displacement.y, 0.0);
    }
}
