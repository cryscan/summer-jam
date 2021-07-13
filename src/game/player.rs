use crate::{
    game::{ball::Ball, rigid_body::*},
    states::{make_ball, GameOverEvent},
    utils::Damp,
};
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::{error::Error, ops::Add};

pub struct Player {
    pub speed_limit: f32,
    pub speed: f32,
    pub damp: f32,
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
                delta * player.speed / time.delta_seconds(),
                player.damp,
                time.delta_seconds(),
            )
            .clamp_length_max(player.speed_limit);
    }
}

#[derive(new)]
pub struct PlayerBase {
    pub lives: i32,
}

#[derive(new)]
pub struct EnemyBase {
    pub hp: f32,
}

pub fn player_goal(
    mut collision_events: EventReader<CollisionEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    mut query: QuerySet<(Query<&RigidBody, With<Ball>>, Query<&mut EnemyBase>)>,
) {
    for event in collision_events.iter() {
        let mut resolve =
            |ball_entity: Entity, base_entity: Entity| -> Result<(), Box<dyn Error>> {
                let mass = query.q0().get(ball_entity)?.mass;
                let mut base = query.q1_mut().get_mut(base_entity)?;

                if base.hp < 0.0 {
                    game_over_events.send(GameOverEvent::Win);
                } else {
                    base.hp -= event.speed * mass;
                }

                Ok(())
            };

        resolve(event.first, event.second).unwrap_or_default();
        resolve(event.second, event.first).unwrap_or_default();
    }
}

pub fn player_miss(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    mut query: QuerySet<(Query<&Ball>, Query<&mut PlayerBase>)>,
) {
    let mut resolve = |ball_entity: Entity, base_entity: Entity| -> Result<(), Box<dyn Error>> {
        let _ball = query.q0().get(ball_entity)?;
        let mut base = query.q1_mut().get_mut(base_entity)?;

        if base.lives == 0 {
            game_over_events.send(GameOverEvent::Lose);
        } else {
            base.lives -= 1;
        }

        commands.entity(ball_entity).despawn();

        Ok(())
    };

    for event in collision_events.iter() {
        resolve(event.first, event.second).unwrap_or_default();
        resolve(event.second, event.first).unwrap_or_default();
    }
}

pub fn remake_ball(
    commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
    query: Query<&Ball>,
) {
    if query.iter().count() == 0 {
        make_ball(commands, asset_server, materials);
    }
}
