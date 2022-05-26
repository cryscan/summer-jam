use super::{
    ball::{Ball, Point, Trajectory},
    enemy::Controller,
    physics::{Motion, RigidBody},
};
use crate::{
    config::ARENA_HEIGHT,
    utils::{Damp, TimeScale},
};
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::ops::Add;

#[derive(Component)]
pub struct Player {
    pub max_speed: f32,
    pub sensitivity: f32,
    pub damp: f32,

    pub assist_speed: f32,
    pub assist_speed_threshold: f32,
}

pub fn player_movement(
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&Player, Option<&Controller>, &mut Motion)>,
) {
    let (player, controller, mut motion) = query.single_mut();
    let delta = mouse_motion_events
        .iter()
        .map(|mouse_motion| mouse_motion.delta)
        .map(|v| Vec2::new(v.x, -v.y))
        .fold(Vec2::ZERO, Vec2::add);

    let delta_seconds = time.delta_seconds() * time_scale.0;

    let velocity = delta * player.sensitivity / delta_seconds
        + controller.map_or(Vec2::ZERO, |controller| controller.velocity);

    motion.velocity = motion
        .velocity
        .damp(velocity, player.damp, delta_seconds)
        .clamp_length_max(player.max_speed);
}

pub fn player_assist(
    time: Res<Time>,
    mut time_scale: ResMut<TimeScale>,
    mut query: Query<(&Transform, &RigidBody, &Player, &mut Controller), Without<Ball>>,
    ball_query: Query<(&Motion, &Trajectory), With<Ball>>,
) {
    let (transform, rigid_body, player, mut controller) = query.single_mut();
    controller.velocity = Vec2::ZERO;
    let width = rigid_body.size.x;

    for (motion, trajectory) in ball_query.iter() {
        let position = transform.translation.truncate();

        if motion.velocity.y < player.assist_speed_threshold
            && (motion.translation.x - transform.translation.x).abs() > width / 2.0
        {
            // very dangerous, try to assist the player
            let delta_seconds = time.seconds_since_startup() - trajectory.start_time;
            if let Some(candidate) = trajectory
                .points
                .iter()
                .filter(|point| point.position.y < 0.0)
                .filter(|point| point.position.y > -ARENA_HEIGHT / 2.0 + 16.0)
                .filter(|point| point.velocity.y < 0.0)
                .max_by(|a, b| {
                    let cost = |point: &Point| {
                        // space-time cost
                        let time = (point.time - delta_seconds) as f32;
                        let distance = (point.position - position).length();
                        time - distance / player.assist_speed
                    };
                    cost(a)
                        .partial_cmp(&cost(b))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                let direction = candidate.position - position;
                let time = (candidate.time - delta_seconds) as f32;
                let distance = direction.length();

                let mut speed = (1.5 * (distance / time + 1.0)).clamp(0.0, player.assist_speed);

                let stop_distance = 1.5 * width;
                if distance < stop_distance {
                    speed *= distance / stop_distance;
                }
                controller.velocity = speed * direction.normalize();
            }
        }

        let mut target_time_scale: f32 = 1.0;
        if motion.velocity.y < player.assist_speed_threshold {
            let delta = motion.translation.y - transform.translation.y - ARENA_HEIGHT / 8.0;
            target_time_scale = target_time_scale.min(delta / ARENA_HEIGHT * 2.0).max(0.2);
        }
        time_scale.0 = time_scale
            .0
            .damp(target_time_scale, 100.0, time.delta_seconds());
    }
}
