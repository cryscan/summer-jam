use crate::{
    config::*,
    game::{ball::*, player::*, rigid_body::*},
    AppState,
};
use bevy::prelude::*;
use std::error::Error;

pub enum GameOverEvent {
    Win,
    Lose,
}

struct GameStateTag;

struct Materials {
    pub player_material: Handle<ColorMaterial>,
    pub ball_material: Handle<ColorMaterial>,
    pub boundary_material: Handle<ColorMaterial>,
    pub separate_material: Handle<ColorMaterial>,
}

#[derive(new)]
pub struct PlayerBase {
    pub lives: i32,
}

#[derive(new)]
pub struct EnemyBase {
    pub hp: f32,
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Entering Game");

    commands.insert_resource(Materials {
        player_material: materials.add(asset_server.load(PLAYER_SPRITE).into()),
        ball_material: materials.add(asset_server.load(BALL_SPRITE).into()),
        boundary_material: materials.add(Color::NONE.into()),
        separate_material: materials.add(Color::rgba(0.5, 0.5, 0.5, 0.1).into()),
    });
}

fn update_game(
    mut app_state: ResMut<State<AppState>>,
    mut input: ResMut<Input<KeyCode>>,
    mut events: EventReader<GameOverEvent>,
) {
    if input.just_pressed(KeyCode::Escape) {
        input.reset(KeyCode::Escape);
        app_state.set(AppState::Title).unwrap();
    }

    for event in events.iter() {
        match event {
            GameOverEvent::Win => app_state.set(AppState::Title).unwrap(),
            GameOverEvent::Lose => app_state.set(AppState::Title).unwrap(),
        }
    }
}

fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameStateTag>>) {
    println!("Cleaning-up Title");

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn make_static_entities(mut commands: Commands, materials: Res<Materials>) {
    // middle Separate
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.separate_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(ARENA_WIDTH, 16.0)),
            ..Default::default()
        })
        .insert(GameStateTag)
        .insert(RigidBody::new(Layer::Separate, 1.0, 0.9, 0.5));

    // top boundary
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.boundary_material.clone(),
            transform: Transform::from_xyz(0.0, ARENA_HEIGHT / 2.0 + 16.0, 0.0),
            sprite: Sprite::new(Vec2::new(ARENA_WIDTH, 32.0)),
            ..Default::default()
        })
        .insert(GameStateTag)
        .insert(EnemyBase::new(10000.0))
        .insert(RigidBody::new(Layer::Boundary, 1.0, 0.9, 0.5));

    // bottom boundary
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.boundary_material.clone(),
            transform: Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0 - 16.0, 0.0),
            sprite: Sprite::new(Vec2::new(ARENA_WIDTH, 32.0)),
            ..Default::default()
        })
        .insert(GameStateTag)
        .insert(PlayerBase::new(3))
        .insert(RigidBody::new(Layer::Boundary, 1.0, 0.9, 0.5));

    // left boundary
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.boundary_material.clone(),
            transform: Transform::from_xyz(-ARENA_WIDTH / 2.0 - 16.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(32.0, ARENA_HEIGHT)),
            ..Default::default()
        })
        .insert(GameStateTag)
        .insert(RigidBody::new(Layer::Boundary, 1.0, 0.9, 0.5));

    // right boundary
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.boundary_material.clone(),
            transform: Transform::from_xyz(ARENA_WIDTH / 2.0 + 16.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(32.0, ARENA_HEIGHT)),
            ..Default::default()
        })
        .insert(GameStateTag)
        .insert(RigidBody::new(Layer::Boundary, 1.0, 0.9, 0.5));
}

fn make_player(mut commands: Commands, materials: Res<Materials>) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.player_material.clone(),
            transform: Transform::from_xyz(0.0, -160.0, 0.0),
            sprite: Sprite::new(Vec2::new(64.0, 16.0)),
            ..Default::default()
        })
        .insert(GameStateTag)
        .insert(Player {
            speed_limit: 1000.0,
            speed: 0.5,
            damp: 20.0,
        })
        .insert(RigidBody::new(Layer::Player, 4.0, 0.9, 1.0))
        .insert(Motion::default());
}

fn make_ball(mut commands: Commands, materials: Res<Materials>, query: Query<&Ball>) {
    if query.iter().count() == 0 {
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.ball_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            })
            .insert(GameStateTag)
            .insert(Ball {
                gravity: -1000.0,
                timer: Timer::from_seconds(1.0, false),
            })
            .insert(RigidBody::new(Layer::Ball, 1.0, 0.9, 0.5));
    }
}

fn player_goal(
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

fn player_miss(
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

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(RigidBodyPlugin)
            .add_event::<GameOverEvent>()
            .add_startup_system(setup_game)
            .add_system_set(
                SystemSet::on_enter(AppState::Game)
                    .with_system(make_static_entities)
                    .with_system(make_player),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Game)
                    .with_system(update_game)
                    .with_system(player_movement)
                    .with_system(player_goal)
                    .with_system(player_miss)
                    .with_system(make_ball)
                    .with_system(ball_movement)
                    .with_system(ball_setup),
            )
            .add_system_set(SystemSet::on_exit(AppState::Game).with_system(cleanup_game));
    }
}
