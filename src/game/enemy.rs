use crate::{
    config::{ARENA_HEIGHT, ENEMY_WIDTH},
    game::prelude::*,
    utils::Damp,
};
use bevy::prelude::*;

#[derive(new)]
pub struct Enemy {
    max_speed: f32,
    normal_speed: f32,
    damp: f32,
    hit_range: f32,
}

#[derive(new)]
pub struct Controller {
    #[new(default)]
    velocity: Vec2,
    _hit_timer: Timer,
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
    mut controller_query: Query<(&Transform, &Enemy, &mut Controller), Without<Ball>>,
    ball_query: Query<(&Transform, &Trajectory), With<Ball>>,
) {
    for (transform, enemy, mut controller) in controller_query.iter_mut() {
        for (ball_transform, trajectory) in ball_query.iter() {
            let updated_velocity;

            let direction = (ball_transform.translation - transform.translation).truncate();
            let position = transform.translation.truncate();

            if direction.x.abs() < ENEMY_WIDTH / 2.0
                && direction.y > -enemy.hit_range
                && direction.y < 0.0
                && position.y > 0.25 * ARENA_HEIGHT
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
                                + 4.0 * (ARENA_HEIGHT / 2.0 - target.y).powi(2)
                        };
                        cost(a.position)
                            .partial_cmp(&cost(b.position))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }) {
                    let direction = candidate.position - position;
                    let speed = if direction.length() < ENEMY_WIDTH {
                        enemy.normal_speed * direction.length() / ENEMY_WIDTH
                    } else {
                        enemy.normal_speed
                    };
                    speed * direction.normalize()
                } else {
                    let collection: Vec<_> = trajectory
                        .points
                        .iter()
                        .map(|point| point.position)
                        .filter(|position| position.y > -ARENA_HEIGHT / 2.0)
                        .map(|position| {
                            if position.y < 0.0 {
                                position
                            } else {
                                Vec2::new(-position.x, position.y)
                            }
                        })
                        .collect();

                    let mut candidate = collection.iter().sum::<Vec2>() / collection.len() as f32;
                    candidate.y = -candidate.y;
                    candidate.y = candidate
                        .y
                        .clamp(0.125 * ARENA_HEIGHT, 0.375 * ARENA_HEIGHT);

                    let direction = candidate - position;
                    let speed = if direction.length() < ENEMY_WIDTH {
                        enemy.normal_speed * direction.length() / ENEMY_WIDTH
                    } else {
                        enemy.normal_speed
                    };

                    speed * direction.normalize()
                }
            }

            controller.velocity = updated_velocity;
        }
    }
}
