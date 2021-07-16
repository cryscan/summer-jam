use crate::{game::prelude::*, utils::Damp};
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::ops::Add;

#[derive(new)]
pub struct Player {
    max_speed: f32,
    sensitivity: f32,
    damp: f32,
}

pub fn player_movement(
    time: Res<Time>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&Player, &mut Motion)>,
) {
    if let Ok((player, mut motion)) = query.single_mut() {
        let delta = mouse_motion_events
            .iter()
            .map(|mouse_motion| mouse_motion.delta)
            .map(|v| Vec2::new(v.x, -v.y))
            .fold(Vec2::ZERO, Vec2::add);

        motion.velocity = motion
            .velocity
            .damp(
                delta * player.sensitivity / time.delta_seconds(),
                player.damp,
                time.delta_seconds(),
            )
            .clamp_length_max(player.max_speed);
    }
}
