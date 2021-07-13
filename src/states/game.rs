use crate::{
    config::*,
    game::{ball::*, player::*, rigid_body::*},
    AppState,
};
use bevy::prelude::*;

pub enum GameOverEvent {
    Win,
    Lose,
}

struct GameEntity;

fn setup_game(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    println!("Entering Game");

    // top boundary
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(0.0, ARENA_HEIGHT / 2.0 + 16.0, 0.0),
            sprite: Sprite::new(Vec2::new(ARENA_WIDTH, 32.0)),
            ..Default::default()
        })
        .insert(GameEntity)
        .insert(RigidBody::new(Layer::Boundary, 1.0, 0.9, 0.5));

    // bottom boundary
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0 - 16.0, 0.0),
            sprite: Sprite::new(Vec2::new(ARENA_WIDTH, 32.0)),
            ..Default::default()
        })
        .insert(GameEntity)
        .insert(PlayerBase { lives: 3 })
        .insert(RigidBody::new(Layer::Boundary, 1.0, 0.9, 0.5));

    // left boundary
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(-ARENA_WIDTH / 2.0 - 16.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(32.0, ARENA_HEIGHT)),
            ..Default::default()
        })
        .insert(GameEntity)
        .insert(RigidBody::new(Layer::Boundary, 1.0, 0.9, 0.5));

    // right boundary
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_xyz(ARENA_WIDTH / 2.0 + 16.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(32.0, ARENA_HEIGHT)),
            ..Default::default()
        })
        .insert(GameEntity)
        .insert(RigidBody::new(Layer::Boundary, 1.0, 0.9, 0.5));
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

fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
    println!("Cleaning-up Title");

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn make_player(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
            transform: Transform::from_xyz(0.0, -160.0, 0.0),
            sprite: Sprite::new(Vec2::new(64.0, 16.0)),
            ..Default::default()
        })
        .insert(GameEntity)
        .insert(Player {
            speed_limit: 1000.0,
            speed: 0.5,
            damp: 20.0,
        })
        .insert(RigidBody::new(Layer::Player, 4.0, 0.9, 1.0))
        .insert(Motion::default());
}

pub fn make_ball(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(asset_server.load(BALL_SPRITE).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(GameEntity)
        .insert(Ball {
            gravity: -1000.0,
            timer: Timer::from_seconds(1.0, false),
        })
        .insert(RigidBody::new(Layer::Ball, 1.0, 0.9, 0.5));
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(RigidBodyPlugin)
            .add_event::<GameOverEvent>()
            .add_system_set(
                SystemSet::on_enter(AppState::Game)
                    .with_system(setup_game)
                    .with_system(make_player)
                    .with_system(make_ball),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Game)
                    .with_system(update_game)
                    .with_system(player_movement)
                    .with_system(player_miss)
                    .with_system(remake_ball)
                    .with_system(ball_movement)
                    .with_system(ball_setup),
            )
            .add_system_set(SystemSet::on_exit(AppState::Game).with_system(cleanup_game));
    }
}
