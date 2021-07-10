use crate::{game::rigid_body::RigidBody, utils::Damp};
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::ops::Add;

pub struct Player {
    pub speed_limit: f32,
    pub speed: f32,
    pub damp: f32,
}

pub fn player_movement(
    time: Res<Time>,
    mut mouse_motion_event: EventReader<MouseMotion>,
    mut query: Query<(&Player, &mut RigidBody)>,
) {
    if let Ok((player, mut rigid_body)) = query.single_mut() {
        let delta = mouse_motion_event
            .iter()
            .map(|mouse_motion| mouse_motion.delta)
            .map(|v| Vec2::new(v.x, -v.y))
            .fold(Vec2::ZERO, Vec2::add);

        rigid_body.velocity = rigid_body
            .velocity
            .damp(
                delta * player.speed / time.delta_seconds(),
                player.damp,
                time.delta_seconds(),
            )
            .clamp_length_max(player.speed_limit);
    }
}
