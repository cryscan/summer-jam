use super::rigid_body::CollisionEvent;
use crate::{game::rigid_body::RigidBody, utils::Damp};
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::ops::Add;

pub struct Player {
    pub speed_limit: f32,
    pub speed: f32,
    pub damp: f32,

    pub recover_timer: Timer,
}

pub fn player_movement(
    time: Res<Time>,
    mut collision_event: EventReader<CollisionEvent>,
    mut mouse_motion_event: EventReader<MouseMotion>,
    mut query: Query<(Entity, &mut Player, &mut RigidBody)>,
) {
    if let Ok((entity, mut player, mut rigid_body)) = query.single_mut() {
        for event in collision_event.iter() {
            if event.impact > 10.0 && (event.first == entity || event.second == entity) {
                player.recover_timer.reset();
            }
        }

        let delta = if player.recover_timer.tick(time.delta()).finished() {
            mouse_motion_event
                .iter()
                .map(|mouse_motion| mouse_motion.delta)
                .map(|v| Vec2::new(v.x, -v.y))
                .fold(Vec2::ZERO, Vec2::add)
        } else {
            Vec2::ZERO
        };

        rigid_body.velocity = rigid_body.velocity.damp(
            (delta * player.speed / time.delta_seconds()).clamp_length_max(player.speed_limit),
            player.damp,
            time.delta_seconds(),
        );
    }
}
