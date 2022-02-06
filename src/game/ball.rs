use super::physics::{Motion, RigidBody};
use crate::config::*;
use bevy::prelude::*;

#[derive(Clone, Component)]
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

pub fn ball_predict(
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
                velocity.x = -rigid_body.bounciness * velocity.x;
                velocity.y = rigid_body.friction * velocity.y;
                position.x = position.x.clamp(-boundary.x + 0.01, boundary.x - 0.01);
            }

            if position.y > boundary.y {
                velocity.y = -rigid_body.bounciness * velocity.y;
                velocity.x = rigid_body.friction * velocity.x;
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
