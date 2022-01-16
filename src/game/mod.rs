use self::{ball::*, base::*, enemy::*, hint::*, physics::*, player::*};
use crate::{
    config::*,
    score::Score,
    utils::{cleanup_system, Interpolation},
    AppState,
};
use bevy::{core::FixedTimestep, prelude::*};
use bevy_kira_audio::{Audio, AudioChannel, AudioSource};
use itertools::Itertools;
use std::error::Error;

mod ball;
mod base;
mod enemy;
mod hint;
mod physics;
mod player;

enum GameOverEvent {
    Win,
    Lose,
}

struct PlayerHitEvent(f32);
struct PlayerMissEvent;

struct DebounceTimer(Timer);

#[derive(Component)]
struct BounceAudio;

#[derive(Component)]
struct Cleanup;

struct Materials {
    // dynamic entities
    player: Handle<Image>,
    enemy: Handle<Image>,
    paddle: Color,
    ball: Handle<Image>,
    hint: Handle<Image>,

    // static entities
    boundary: Color,
    base: Color,
    separate: Color,

    // ui
    node: Color,
    health_bar: Color,
    health_bar_tracker: Color,
}

struct Audios {
    hit_audio: Handle<AudioSource>,
    miss_audio: Handle<AudioSource>,
    explosion_audio: Handle<AudioSource>,
    lose_audio: Handle<AudioSource>,
    impact_audios: Vec<Handle<AudioSource>>,
}

fn setup_game(mut commands: Commands, time: Res<Time>, asset_server: Res<AssetServer>) {
    commands.insert_resource(Materials {
        player: asset_server.load(PLAYER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        paddle: Color::rgba_u8(155, 173, 183, 50),
        ball: asset_server.load(BALL_SPRITE),
        hint: asset_server.load(HINT_SPRITE),

        boundary: Color::WHITE,
        base: Color::rgb_u8(155, 173, 183),
        separate: Color::rgba(0.5, 0.5, 0.5, 0.1),

        node: Color::NONE,
        health_bar: Color::rgb_u8(155, 173, 183),
        health_bar_tracker: Color::rgb_u8(217, 87, 99),
    });

    commands.insert_resource(Audios {
        hit_audio: asset_server.load(HIT_AUDIO),
        miss_audio: asset_server.load(MISS_AUDIO),
        explosion_audio: asset_server.load(EXPLOSION_AUDIO),
        lose_audio: asset_server.load(LOSE_AUDIO),
        impact_audios: IMPACT_AUDIOS
            .iter()
            .map(|path| asset_server.load(*path))
            .collect_vec(),
    });

    commands.insert_resource(Score {
        timestamp: time.seconds_since_startup(),
        hits: 0,
        miss: 0,
    });
}

fn enter_game(time: Res<Time>, mut score: ResMut<Score>) {
    println!("Entering Game");

    // clear score state
    score.timestamp = time.seconds_since_startup();
    score.hits = 0;
    score.miss = 0;
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
            GameOverEvent::Win => app_state.set(AppState::Win).unwrap(),
            GameOverEvent::Lose => app_state.set(AppState::Title).unwrap(),
        }
    }
}

fn make_static_entities(mut commands: Commands, materials: Res<Materials>) {
    // middle Separate
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite {
                color: materials.separate,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 16.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(RigidBody::new(
            Layer::Separate,
            Vec2::new(ARENA_WIDTH, 16.0),
            0.0,
            0.9,
            0.5,
        ));

    // top boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, ARENA_HEIGHT / 2.0 + 8.0, 0.0),
            sprite: Sprite {
                color: materials.base,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 16.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(EnemyBase::new(10000.0, 10000.0))
        .insert(RigidBody::new(
            Layer::Boundary,
            Vec2::new(ARENA_WIDTH, 16.0),
            0.0,
            0.9,
            0.0,
        ));

    // bottom boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0 - 8.0, 0.0),
            sprite: Sprite {
                color: materials.boundary,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 16.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(PlayerBase::new(3))
        .insert(RigidBody::new(
            Layer::Boundary,
            Vec2::new(ARENA_WIDTH, 16.0),
            0.0,
            0.9,
            0.5,
        ));

    // left boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(-ARENA_WIDTH / 2.0 - 8.0, 0.0, 0.0),
            sprite: Sprite {
                color: materials.boundary,
                custom_size: Some(Vec2::new(16.0, ARENA_HEIGHT + 32.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(RigidBody::new(
            Layer::Boundary,
            Vec2::new(16.0, ARENA_HEIGHT + 32.0),
            0.0,
            1.0,
            0.0,
        ));

    // right boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(ARENA_WIDTH / 2.0 + 8.0, 0.0, 0.0),
            sprite: Sprite {
                color: materials.boundary,
                custom_size: Some(Vec2::new(16.0, ARENA_HEIGHT + 32.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(RigidBody::new(
            Layer::Boundary,
            Vec2::new(16.0, ARENA_HEIGHT + 32.0),
            0.0,
            1.0,
            0.0,
        ));
}

fn make_ui(mut commands: Commands, materials: Res<Materials>, asset_server: Res<AssetServer>) {
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
            color: materials.node.into(),
            ..Default::default()
        })
        .insert(Cleanup)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    color: materials.health_bar.into(),
                    ..Default::default()
                })
                .insert(HealthBar);
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    color: materials.health_bar_tracker.into(),
                    ..Default::default()
                })
                .insert(HealthBarTracker::new(1.0, 10.0));
        });

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(16.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: materials.node.into(),
            ..Default::default()
        })
        .insert(Cleanup)
        .with_children(|parent| {
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(16.0), Val::Px(16.0)),
                    ..Default::default()
                },
                image: materials.ball.clone().into(),
                ..Default::default()
            });

            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: " x ".into(),
                                style: TextStyle {
                                    font: asset_server.load(FONT_FIRA_MONO),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "".into(),
                                style: TextStyle {
                                    font: asset_server.load(FONT_FIRA_MONO),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(BallCounter);
        });
}

fn make_player(mut commands: Commands, materials: Res<Materials>) {
    let hint = commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, ARENA_HEIGHT / 2.0, 0.0),
            texture: materials.hint.clone(),
            ..Default::default()
        })
        .insert(Cleanup)
        .id();

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, -160.0, 0.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                color: materials.paddle,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(Player::new(
            PLAYER_MAX_SPEED,
            0.5,
            20.0,
            PLAYER_ASSIST_SPEED,
            PLAYER_ASSIST_SPEED_THRESHOLD,
        ))
        .insert(Controller::new())
        .insert(RigidBody::new(
            Layer::Player,
            Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
            3.0,
            2.0,
            1.0,
        ))
        .insert(Motion::default())
        .insert(Hint(hint))
        .insert(BounceAudio)
        .with_children(|parent| {
            parent.spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(-PADDLE_WIDTH / 2.0 + 8.0, 0.0, 0.1),
                texture: materials.player.clone(),
                ..Default::default()
            });

            parent.spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(PADDLE_WIDTH / 2.0 - 8.0, 0.0, 0.1),
                texture: materials.player.clone(),
                ..Default::default()
            });
        });
}

fn make_enemy(mut commands: Commands, materials: Res<Materials>) {
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, 160.0, 0.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                color: materials.paddle,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(Enemy::new(
            ENEMY_MIN_SPEED,
            ENEMY_MAX_SPEED,
            ENEMY_NORMAL_SPEED,
            20.0,
            PADDLE_WIDTH,
            ENEMY_HIT_SPEED_THRESHOLD,
            0.125 * ARENA_HEIGHT,
        ))
        .insert(Controller::new())
        .insert(RigidBody::new(
            Layer::Player,
            Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
            3.0,
            1.0,
            1.0,
        ))
        .insert(Motion::default())
        .insert(BounceAudio)
        .with_children(|parent| {
            parent.spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(-PADDLE_WIDTH / 2.0 + 8.0, 0.0, 0.1),
                texture: materials.enemy.clone(),
                ..Default::default()
            });

            parent.spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(PADDLE_WIDTH / 2.0 - 8.0, 0.0, 0.1),
                texture: materials.enemy.clone(),
                ..Default::default()
            });
        });
}

fn make_ball(mut commands: Commands, materials: Res<Materials>, query: Query<&Ball>) {
    if query.iter().count() == 0 {
        let hint = commands
            .spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0, 0.0),
                texture: materials.hint.clone(),
                ..Default::default()
            })
            .insert(Cleanup)
            .id();

        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                texture: materials.ball.clone(),
                ..Default::default()
            })
            .insert(Cleanup)
            .insert(Ball::new(-1000.0, Timer::from_seconds(1.0, false)))
            .insert(RigidBody::new(
                Layer::Ball,
                Vec2::new(BALL_SIZE, BALL_SIZE),
                1.0,
                1.0,
                0.5,
            ))
            .insert(Trajectory {
                start_time: 0.0,
                points: vec![Point::default(); PREDICT_SIZE],
            })
            .insert(Hint(hint))
            .insert(BounceAudio);
    }
}

fn player_hit(
    mut collision_events: EventReader<CollisionEvent>,
    mut player_hit_events: EventWriter<PlayerHitEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    ball_query: Query<(&RigidBody, &Motion), With<Ball>>,
    mut base_query: Query<&mut EnemyBase>,
) {
    for event in collision_events.iter() {
        let mut closure =
            |ball_entity: Entity, base_entity: Entity| -> Result<(), Box<dyn Error>> {
                let (rigid_body, motion) = ball_query.get(ball_entity)?;
                let mass = rigid_body.mass();
                let speed = motion.velocity.length();

                let mut base = base_query.get_mut(base_entity)?;
                if base.hp <= 0.0 {
                    game_over_events.send(GameOverEvent::Win);
                } else {
                    let damage = base.hp.min(speed * mass).min(MAX_DAMAGE);
                    base.hp -= damage;
                    player_hit_events.send(PlayerHitEvent(damage));
                }

                Ok(())
            };

        closure(event.first, event.second)
            .unwrap_or_else(|_| closure(event.second, event.first).unwrap_or_default())
    }
}

fn player_miss(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_miss_events: EventWriter<PlayerMissEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    ball_query: Query<Option<&Hint>, With<Ball>>,
    mut base_query: Query<(Entity, &mut PlayerBase), Without<Ball>>,
) {
    let (entity, mut base) = base_query.single_mut();

    let mut closure = |ball_entity: Entity, base_entity: Entity| {
        if base_entity != entity {
            return;
        }

        if let Ok(hint) = ball_query.get(ball_entity) {
            if base.balls == 0 {
                game_over_events.send(GameOverEvent::Lose);
            } else {
                base.balls -= 1;
                player_miss_events.send(PlayerMissEvent);
            }

            if let Some(hint) = hint {
                commands.entity(hint.0).despawn();
            }
            commands.entity(ball_entity).despawn();
        }
    };

    for event in collision_events.iter() {
        closure(event.first, event.second);
        closure(event.second, event.first);
    }
}

fn score_system(
    mut player_hit_events: EventReader<PlayerHitEvent>,
    mut player_miss_events: EventReader<PlayerMissEvent>,
    mut score: ResMut<Score>,
) {
    for _ in player_hit_events.iter() {
        score.hits += 1;
    }

    for _ in player_miss_events.iter() {
        score.miss += 1;
    }
}

fn bounce_audio(
    audios: Res<Audios>,
    audio: Res<Audio>,
    time: Res<Time>,
    mut timer: ResMut<DebounceTimer>,
    mut events: EventReader<CollisionEvent>,
    mut index: Local<u32>,
    query: Query<(), With<BounceAudio>>,
) {
    let can_play_audio = timer.0.tick(time.delta()).finished();

    for event in events.iter() {
        let ref channel = AudioChannel::new(format!("impact-{}", *index).into());
        *index = (*index + 1) % MAX_IMPACT_AUDIO_CHANNELS;

        let mut closure = |first: Entity, second: Entity| {
            if query.get(first).and_then(|_| query.get(second)).is_err() {
                return;
            }

            let speed = event.velocity.length();
            if speed > MIN_BOUNCE_AUDIO_SPEED {
                let normalized_speed = speed
                    .intermediate(MIN_BOUNCE_AUDIO_SPEED, MAX_BOUNCE_AUDIO_SPEED)
                    .clamp(0.0, 1.0);

                let volume = 0.2 * normalized_speed + 0.05;
                audio.set_volume_in_channel(volume, channel);

                let pitch = ((normalized_speed * 4.0) as usize).min(3);
                let audio_source = audios.impact_audios[pitch].clone();
                audio.play_in_channel(audio_source, channel);

                timer.0.reset();
            }
        };

        if can_play_audio {
            closure(event.first, event.second);
        }
    }
}

fn score_audio(
    audios: Res<Audios>,
    audio: Res<Audio>,
    mut player_hit_events: EventReader<PlayerHitEvent>,
    mut player_miss_events: EventReader<PlayerMissEvent>,
    mut game_over_events: EventReader<GameOverEvent>,
) {
    for _ in player_hit_events.iter() {
        audio.play(audios.hit_audio.clone());
    }

    for _ in player_miss_events.iter() {
        audio.play(audios.miss_audio.clone());
    }

    for event in game_over_events.iter() {
        match event {
            GameOverEvent::Win => audio.play(audios.explosion_audio.clone()),
            GameOverEvent::Lose => audio.play(audios.lose_audio.clone()),
        };
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameOverEvent>()
            .add_event::<PlayerHitEvent>()
            .add_event::<PlayerMissEvent>()
            .insert_resource(DebounceTimer(Timer::from_seconds(0.1, false)))
            .add_startup_system(setup_game)
            .add_system_set(
                SystemSet::on_enter(AppState::Game)
                    .with_system(enter_game)
                    .with_system(make_static_entities)
                    .with_system(make_ui)
                    .with_system(make_player)
                    .with_system(make_enemy),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Game)
                    .with_system(update_game)
                    .with_system(make_ball)
                    .with_system(player_movement)
                    .with_system(player_assist)
                    .with_system(enemy_movement)
                    .with_system(player_hit)
                    .with_system(player_miss)
                    .with_system(ball_counter)
                    .with_system(health_bar)
                    .with_system(health_bar_tracker)
                    .with_system(ball_movement)
                    .with_system(ball_setup)
                    .with_system(ball_predict_debug)
                    .with_system(score_system)
                    .with_system(hint_system),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Game).with_system(cleanup_system::<Cleanup>),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(AI_TIME_STEP))
                    .with_system(ball_predict)
                    .with_system(enemy_controller),
            )
            .add_system_set(
                SystemSet::new()
                    .with_system(bounce_audio)
                    .with_system(score_audio),
            )
            .add_plugin(PhysicsPlugin);
    }
}
