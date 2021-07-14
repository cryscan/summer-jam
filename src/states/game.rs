use crate::{
    config::*,
    game::{ball::*, base::*, player::*, rigid_body::*},
    AppState,
};
use bevy::{math::f32, prelude::*};
use std::error::Error;

pub enum GameOverEvent {
    Win,
    Lose,
}

pub struct PlayerHitEvent(pub f32);
pub struct PlayerMissEvent;

struct GameStateTag;

struct Materials {
    player_material: Handle<ColorMaterial>,
    ball_material: Handle<ColorMaterial>,
    boundary_material: Handle<ColorMaterial>,
    separate_material: Handle<ColorMaterial>,

    node_material: Handle<ColorMaterial>,
    health_bar_material: Handle<ColorMaterial>,
    health_bar_tracker_material: Handle<ColorMaterial>,
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

        node_material: materials.add(Color::NONE.into()),
        health_bar_material: materials.add(Color::rgb_u8(155, 173, 183).into()),
        health_bar_tracker_material: materials.add(Color::rgb_u8(217, 87, 99).into()),
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
        commands.entity(entity).despawn_recursive();
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
        .insert(EnemyBase::new(10000.0, 10000.0))
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

fn make_ui(mut commands: Commands, materials: Res<Materials>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(4.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: materials.node_material.clone(),
            ..Default::default()
        })
        .insert(GameStateTag)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    material: materials.health_bar_material.clone(),
                    ..Default::default()
                })
                .insert(HealthBar);
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    material: materials.health_bar_tracker_material.clone(),
                    ..Default::default()
                })
                .insert(HealthBarTracker::new(1.0, 10.0));
        });
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
        .insert(Player::new(1000.0, 0.5, 20.0))
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
            .insert(Ball::new(-1000.0, Timer::from_seconds(1.0, false)))
            .insert(RigidBody::new(Layer::Ball, 1.0, 0.9, 0.5));
    }
}

fn player_hit(
    mut collision_events: EventReader<CollisionEvent>,
    mut player_hit_events: EventWriter<PlayerHitEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    ball_query: Query<&RigidBody, With<Ball>>,
    mut base_query: Query<&mut EnemyBase>,
) {
    for event in collision_events.iter() {
        let mut resolve =
            |ball_entity: Entity, base_entity: Entity| -> Result<(), Box<dyn Error>> {
                let mass = ball_query.get(ball_entity)?.mass;
                let mut base = base_query.get_mut(base_entity)?;

                if base.hp <= 0.0 {
                    game_over_events.send(GameOverEvent::Win);
                } else {
                    let hit = base.hp.min(event.speed * mass);
                    base.hp -= hit;
                    player_hit_events.send(PlayerHitEvent(hit));
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
    mut player_miss_events: EventWriter<PlayerMissEvent>,
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
            player_miss_events.send(PlayerMissEvent);
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
            .add_event::<PlayerHitEvent>()
            .add_event::<PlayerMissEvent>()
            .add_startup_system(setup_game)
            .add_system_set(
                SystemSet::on_enter(AppState::Game)
                    .with_system(make_static_entities)
                    .with_system(make_ui)
                    .with_system(make_player),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Game)
                    .with_system(update_game)
                    .with_system(player_movement)
                    .with_system(player_hit)
                    .with_system(player_miss)
                    .with_system(health_bar)
                    .with_system(make_ball)
                    .with_system(ball_movement)
                    .with_system(ball_setup),
            )
            .add_system_set(SystemSet::on_exit(AppState::Game).with_system(cleanup_game));
    }
}
