use super::{
    ball::{Ball, Point, Trajectory},
    enemy::Controller,
    physics::{CollisionEvent, Motion},
};
use crate::{config::*, utils::Damp, TimeScale};
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::{ops::Add, time::Duration};

#[derive(Component)]
pub struct Player {
    pub max_speed: f32,
    pub sensitivity: f32,
    pub damp: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            max_speed: PLAYER_MAX_SPEED,
            sensitivity: PLAYER_SENSITIVITY,
            damp: PLAYER_DAMP,
        }
    }
}

#[derive(Component)]
pub struct MotionOverride {
    pub timer: Timer,
    pub damp: f32,
}

impl Default for MotionOverride {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.1, TimerMode::Once);
        timer.set_elapsed(Duration::from_secs_f32(0.0));
        Self {
            timer,
            damp: PLAYER_DAMP_MOTION_OVERRIDE,
        }
    }
}

#[derive(Component)]
pub struct PlayerAssist {
    pub range: f32,
    pub speed: f32,
    pub vertical_speed_threshold: f32,
    pub speed_threshold: f32,
}

impl Default for PlayerAssist {
    fn default() -> Self {
        Self {
            range: PLAYER_ASSIST_RANGE,
            speed: PLAYER_ASSIST_SPEED,
            vertical_speed_threshold: PLAYER_ASSIST_VERTICAL_SPEED_THRESHOLD,
            speed_threshold: PLAYER_ASSIST_SPEED_THRESHOLD,
        }
    }
}

pub fn move_player(
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&Player, &Controller, &mut MotionOverride, &mut Motion)>,
) {
    let delta = mouse_motion_events
        .iter()
        .map(|mouse_motion| mouse_motion.delta)
        .map(|v| Vec2::new(v.x, -v.y))
        .fold(Vec2::ZERO, Vec2::add);

    let delta_seconds = time.delta_seconds() * time_scale.0;

    for (player, controller, mut motion_override, mut motion) in query.iter_mut() {
        let velocity = delta * player.sensitivity / delta_seconds + controller.velocity;
        let damp = if motion_override.timer.tick(time.delta()).finished() {
            player.damp
        } else {
            motion_override.damp
        };

        motion.velocity = motion
            .velocity
            .damp(velocity, damp, delta_seconds)
            .clamp_length_max(player.max_speed);
    }
}

#[allow(clippy::type_complexity)]
pub fn assist_player(
    time: Res<Time>,
    mut time_scale: ResMut<TimeScale>,
    mut events: EventReader<CollisionEvent>,
    mut query: Query<
        (
            &Transform,
            &PlayerAssist,
            &mut Controller,
            &mut MotionOverride,
        ),
        (With<Player>, Without<Ball>),
    >,
    ball_query: Query<(&Motion, &Trajectory), With<Ball>>,
) {
    for (transform, assist, mut controller, _) in query.iter_mut() {
        controller.velocity = Vec2::ZERO;

        for (motion, trajectory) in ball_query.iter() {
            let position = transform.translation.truncate();
            let delta = motion.translation - transform.translation;

            if motion.velocity.y < assist.vertical_speed_threshold
                && motion.velocity.length() > assist.speed_threshold
                && delta.x.abs() > assist.range
            {
                // very dangerous, try to assist the player
                let delta_seconds = time.elapsed_seconds() - trajectory.start_time;
                if let Some(candidate) = trajectory
                    .points
                    .iter()
                    .filter(|point| point.position.y < 0.0)
                    .filter(|point| point.position.y > -ARENA_HEIGHT / 2.0 + 16.0)
                    .filter(|point| point.velocity.y < 0.0)
                    .max_by(|a, b| {
                        let cost = |point: &Point| {
                            // space-time cost
                            let time = point.time - delta_seconds;
                            let distance = (point.position - position).length();
                            time - distance / assist.speed
                        };
                        cost(a)
                            .partial_cmp(&cost(b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                {
                    let direction = candidate.position - position;
                    let time = candidate.time - delta_seconds;
                    let distance = direction.length();

                    let mut speed = (1.5 * (distance / time + 1.0)).clamp(0.0, assist.speed);

                    let stop_distance = 1.5 * assist.range;
                    if distance < stop_distance {
                        speed *= distance / stop_distance;
                    }
                    controller.velocity = speed * direction.normalize();
                    controller.velocity.y = 0.0;
                }
            }

            let mut target_time_scale: f32 = 1.0;
            if motion.velocity.y < assist.vertical_speed_threshold
                && motion.velocity.length() > assist.speed_threshold
                && delta.y > 0.0
            {
                target_time_scale = target_time_scale
                    .min(delta.y / ARENA_HEIGHT * 2.0 - 0.25)
                    .max(0.2);
            }
            time_scale.0 =
                time_scale
                    .0
                    .damp(target_time_scale, TIME_SCALE_DAMP, time.delta_seconds());
        }
    }

    // vertical impulse compensation
    let mut closure = |e1: Entity, e2: Entity| -> Option<()> {
        let _ = ball_query.get(e1).ok()?;
        let (_, _, _, mut motion_override) = query.get_mut(e2).ok()?;

        motion_override.timer.reset();

        Some(())
    };

    for event in events.iter() {
        closure(event.entities[0], event.entities[1]);
        closure(event.entities[1], event.entities[0]);
    }
}
