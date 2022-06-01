use super::physics::{Motion, RigidBody};
use crate::{config::*, utils::TimeScale};
use bevy::prelude::*;

#[derive(Clone, Component)]
pub struct Ball {
    pub gravity: f32,
    pub set_timer: Timer,
    pub active_timer: Timer,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            gravity: -1000.0,
            set_timer: Timer::from_seconds(1.0, false),
            active_timer: Timer::from_seconds(2.0, false),
        }
    }
}

pub fn activate_ball(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Ball, &mut Transform)>,
) {
    for (entity, mut ball, mut transform) in query.iter_mut() {
        if ball.set_timer.tick(time.delta()).just_finished() {
            transform.translation = Vec3::ZERO;
        }

        if ball.active_timer.tick(time.delta()).just_finished() {
            commands.entity(entity).insert(Motion::default());
        }
    }
}

pub fn move_ball(
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    mut query: Query<(&Ball, &mut Motion)>,
) {
    for (ball, mut motion) in query.iter_mut() {
        motion.velocity.y += ball.gravity * time.delta_seconds() * time_scale.0;

        let speed = motion.velocity.length();
        if speed > BALL_MAX_SPEED {
            motion.velocity = motion.velocity.normalize() * BALL_MAX_SPEED;
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Point {
    pub position: Vec2,
    pub velocity: Vec2,
    pub time: f64,
}

#[derive(Component)]
pub struct Trajectory {
    pub start_time: f64,
    pub points: Vec<Point>,
}

pub fn predict_ball(
    time: Res<Time>,
    mut query: Query<(&Ball, &RigidBody, &Motion, &mut Trajectory)>,
) {
    for (ball, rigid_body, motion, mut trajectory) in query.iter_mut() {
        let start_time = time.seconds_since_startup();
        let boundary = (Vec2::new(ARENA_WIDTH, ARENA_HEIGHT) + rigid_body.size) / 2.0;

        let mut position = motion.translation.truncate();
        let mut velocity = motion.velocity;
        let mut time = 0.0;

        if let Some(point) = trajectory.points.first_mut() {
            *point = Point {
                position,
                velocity,
                time,
            };
        }

        trajectory.start_time = start_time;
        for point in trajectory.points.iter_mut().skip(1) {
            velocity.y += ball.gravity * PREDICT_TIME_STEP as f32;
            position += velocity * PREDICT_TIME_STEP as f32;

            if position.x.abs() > boundary.x {
                velocity.x *= -rigid_body.bounciness;
                velocity.y *= rigid_body.friction;
                position.x = position.x.clamp(-boundary.x + 0.01, boundary.x - 0.01);
            }

            if position.y > boundary.y {
                velocity.y *= -rigid_body.bounciness;
                velocity.x *= rigid_body.friction;
                position.y = position.y.clamp(-boundary.y + 0.01, boundary.y - 0.01);
            }

            time += PREDICT_TIME_STEP;

            *point = Point {
                position,
                velocity,
                time,
            };
        }
    }
}
