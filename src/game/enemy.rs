use super::{
    ball::{Ball, Trajectory},
    physics::{Motion, RigidBody},
};
use crate::{config::ARENA_HEIGHT, utils::Damp};
use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {
    pub min_speed: f32,
    pub max_speed: f32,
    pub normal_speed: f32,
    pub damp: f32,

    pub hit_range: f32,
    pub hit_speed_threshold: f32,
    pub hit_height_threshold: f32,
}

#[derive(new, Component)]
pub struct Controller {
    #[new(default)]
    pub velocity: Vec2,
}

pub fn enemy_movement(
    time: Res<Time>,
    mut query: Query<(&Enemy, Option<&Controller>, &mut Motion)>,
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

pub fn enemy_controller(
    time: Res<Time>,
    mut query: Query<(&Transform, &RigidBody, &Enemy, &mut Controller), Without<Ball>>,
    ball_query: Query<(&Transform, &Motion, &Trajectory), With<Ball>>,
) {
    for (transform, rigid_body, enemy, mut controller) in query.iter_mut() {
        controller.velocity = Vec2::ZERO;
        let width = rigid_body.size.x;

        for (ball_transform, motion, trajectory) in ball_query.iter() {
            let updated_velocity;

            let direction = (ball_transform.translation - transform.translation).truncate();
            let position = transform.translation.truncate();

            if direction.x.abs() < width / 2.0
                && direction.y > -enemy.hit_range
                && direction.y < -0.0
                && motion.velocity.y > enemy.hit_speed_threshold
                && position.y > enemy.hit_height_threshold
            {
                // very close to the ball, reacts in maximum speed
                updated_velocity = enemy.max_speed * direction.normalize();
            } else {
                // find the most suitable trajectory point
                let delta_seconds = time.seconds_since_startup() - trajectory.start_time;

                updated_velocity = if let Some(candidate) = trajectory
                    .points
                    .iter()
                    .filter(|point| point.position.y > 0.0)
                    .filter(|point| point.position.y < ARENA_HEIGHT / 2.0 - 16.0)
                    .filter(|point| point.velocity.y > 0.0)
                    .filter(|point| {
                        // filter reachable points
                        let time = (point.time - delta_seconds) as f32;
                        let distance = (point.position - position).length();
                        time > distance / enemy.normal_speed
                    })
                    .min_by(|a, b| {
                        let cost = |target: Vec2| {
                            (target - position).length_squared()
                                + (ARENA_HEIGHT / 2.0 - target.y).powi(2)
                        };
                        cost(a.position)
                            .partial_cmp(&cost(b.position))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }) {
                    let direction = candidate.position - position;
                    let time = (candidate.time - delta_seconds) as f32;
                    let distance = direction.length();

                    let mut speed =
                        (distance / time + 1.0).clamp(enemy.min_speed, enemy.normal_speed);

                    let stop_distance = 1.5 * width;
                    if distance < stop_distance {
                        speed *= distance / stop_distance;
                    }
                    speed * direction.normalize()
                } else {
                    let collection: Vec<_> = trajectory
                        .points
                        .iter()
                        .map(|point| point.position)
                        .filter(|position| position.y > -ARENA_HEIGHT / 2.0)
                        .map(|position| {
                            let mut contribution = position;
                            if position.y < 0.0 {
                                contribution.y = -position.y
                            } else {
                                contribution.x = -position.x
                            }
                            contribution
                        })
                        .collect();

                    let mut candidate: Vec2 = collection.iter().sum();
                    candidate /= collection.len() as f32;
                    candidate.y = candidate
                        .y
                        .clamp(0.125 * ARENA_HEIGHT, 0.375 * ARENA_HEIGHT);

                    let direction = candidate - position;
                    let speed = if direction.length() < width {
                        enemy.normal_speed * direction.length() / width
                    } else {
                        enemy.normal_speed
                    };

                    speed * direction.normalize()
                }
            }

            controller.velocity += updated_velocity;
        }
    }
}
