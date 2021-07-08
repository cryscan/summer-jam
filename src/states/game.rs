use crate::{
    game::{player::*, rigid_body::*},
    AppState,
};
use bevy::prelude::*;

struct GameEntity;

fn setup_game(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    println!("Entering Game");

    // Spawn player.
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
            transform: Transform::from_xyz(0.0, -160.0, 0.0),
            sprite: Sprite::new(Vec2::new(32.0, 32.0)),
            ..Default::default()
        })
        .insert(GameEntity)
        .insert(Player {
            speed: 100.0,
            damp: 20.0,
        })
        .insert(RigidBody::new(1.0, 0.5));

    // Spawn ball
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(16.0, 16.0)),
            ..Default::default()
        })
        .insert(GameEntity)
        .insert(RigidBody::new(2.0, 0.5));
}

fn update_game(mut app_state: ResMut<State<AppState>>, mut input: ResMut<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        input.update();
        app_state.set(AppState::Title).unwrap();
    }
}

fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
    println!("Cleaning-up Title");

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(RigidBodyPlugin)
            .add_system_set(SystemSet::on_enter(AppState::Game).with_system(setup_game.system()))
            .add_system_set(
                SystemSet::on_update(AppState::Game)
                    .with_system(update_game.system())
                    .with_system(player_movement.system().before("rigid_body")),
            )
            .add_system_set(SystemSet::on_exit(AppState::Game).with_system(cleanup_game.system()));
    }
}
