use self::{
    ball::*, base::*, battle::*, effects::*, enemy::*, hint::*, physics::*, player::*, practice::*,
    slits::*,
};
use crate::{
    config::*,
    score::Score,
    utils::{cleanup_system, escape_system, Damp, Intermediate},
    AppState, AudioVolume, MusicTrack, TimeScale,
};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep};
use bevy_kira_audio::{
    Audio, AudioApp, AudioChannel, AudioControl, AudioSource, DynamicAudioChannels,
};
use itertools::Itertools;
use std::f32::consts::FRAC_PI_4;

mod ball;
mod base;
mod battle;
mod effects;
mod enemy;
mod hint;
mod physics;
mod player;
mod practice;
mod slits;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameOverEvent>()
            .add_event::<PlayerHitEvent>()
            .add_event::<PlayerMissEvent>()
            .add_event::<BounceEvent>()
            .add_event::<HealEvent>()
            .insert_resource(Debounce {
                audio_bounce_long: Timer::from_seconds(0.5, false),
                audio_bounce_short: Timer::from_seconds(0.1, false),
                audio_hit: Timer::from_seconds(0.1, false),
                bounce: Timer::from_seconds(0.1, false),
                effects: Timer::from_seconds(0.1, false),
                hit: Timer::from_seconds(0.1, false),
                miss: Timer::from_seconds(0.5, false),
            })
            .init_resource::<Slits>()
            .add_audio_channel::<ScoreAudioChannel>()
            .add_startup_system(setup_game)
            .add_system_set(
                SystemSet::new()
                    // fundamental game-play systems
                    .with_system(move_player)
                    .with_system(assist_player)
                    .with_system(move_enemy)
                    .with_system(move_ball)
                    .with_system(activate_ball)
                    .with_system(update_ball)
                    .with_system(ball_bounce)
                    .with_system(heal_enemy_base)
                    .with_system(move_slit_block)
                    .with_system(slits_system)
                    // effects and juice
                    .with_system(bounce_audio)
                    .with_system(score_audio)
                    .with_system(score_effects)
                    .with_system(bounce_effects)
                    // score and display
                    .with_system(count_ball)
                    .with_system(score_system)
                    .with_system(health_bar)
                    .with_system(health_bar_tracker)
                    // hints
                    .with_system(make_player_hint)
                    .with_system(make_ball_hint)
                    .with_system(hint_system),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(AI_TIME_STEP))
                    .with_system(predict_ball)
                    .with_system(control_enemy),
            )
            .add_plugin(PhysicsPlugin)
            .add_plugin(EffectsPlugin)
            .add_plugin(BattlePlugin)
            .add_plugin(PracticePlugin);
    }
}

struct ScoreAudioChannel;

#[derive(Clone, Copy)]
enum GameOverEvent {
    Win,
    Lose,
}

struct PlayerHitEvent {
    ball: Entity,
    location: Vec2,
    win: bool,
}

struct PlayerMissEvent {
    ball: Entity,
    location: Vec2,
    lose: bool,
}

#[allow(dead_code)]
struct BounceEvent {
    ball: Entity,
    other: Entity,
    location: Vec2,
}

struct Debounce {
    audio_bounce_long: Timer,
    audio_bounce_short: Timer,
    audio_hit: Timer,

    bounce: Timer,
    effects: Timer,
    hit: Timer,
    miss: Timer,
}
struct GameOver {
    slow_motion_timer: Timer,
    state_change_timer: Timer,
    event: Option<GameOverEvent>,
}

impl Default for GameOver {
    fn default() -> Self {
        Self {
            slow_motion_timer: Timer::from_seconds(GAME_OVER_SLOW_MOTION_DURATION, false),
            state_change_timer: Timer::from_seconds(GAME_OVER_STATE_CHANGE_DURATION, false),
            event: None,
        }
    }
}

#[derive(Component)]
struct Cleanup;

#[derive(Clone, Copy, PartialEq, Eq, Component)]
enum BounceAudio {
    Bounce,
    Hit,
}

struct Materials {
    player: Handle<Image>,
    enemy: Handle<Image>,
    ball: Handle<Image>,
    hint: Handle<Image>,
    death: Handle<Image>,
    hit: Handle<TextureAtlas>,
}

struct Audios {
    hit_audio: Handle<AudioSource>,
    miss_audio: Handle<AudioSource>,
    explosion_audio: Handle<AudioSource>,
    lose_audio: Handle<AudioSource>,
    impact_audios: Vec<Handle<AudioSource>>,
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.insert_resource(Materials {
        player: asset_server.load(PLAYER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        ball: asset_server.load(BALL_SPRITE),
        hint: asset_server.load(HINT_SPRITE),
        death: asset_server.load(DEATH_SPRITE),
        hit: texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load(HIT_SPRITE),
            Vec2::new(1024.0, 1024.0),
            4,
            4,
        )),
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

    commands.init_resource::<Score>();
}

fn make_arena(mut commands: Commands) {
    // middle Separate
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, 8.0, 0.0),
            sprite: Sprite {
                color: BOUNDARY_COLOR,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 32.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::new(Vec2::new(ARENA_WIDTH, 32.0), 0.0, 0.9, 0.5),
            PhysicsLayers::SEPARATE,
            Cleanup,
        ))
        .with_children(|parent| {
            parent.spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(0.0, -8.0, 0.0),
                sprite: Sprite {
                    color: SEPARATE_COLOR,
                    custom_size: Some(Vec2::new(ARENA_WIDTH, 16.0)),
                    ..Default::default()
                },
                ..Default::default()
            });
        });

    // top boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, ARENA_HEIGHT * 0.5 + 16.0, 0.0),
            sprite: Sprite {
                color: BOUNDARY_COLOR,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 32.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::new(Vec2::new(ARENA_WIDTH, 32.0), 0.0, 0.9, 0.0),
            PhysicsLayers::BOUNDARY,
            BounceAudio::Hit,
            EnemyBase::default(),
            Cleanup,
        ));

    // bottom boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, -ARENA_HEIGHT * 0.5 - 16.0, 0.0),
            sprite: Sprite {
                color: BOUNDARY_COLOR,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 32.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::new(Vec2::new(ARENA_WIDTH, 32.0), 0.0, 0.9, 0.5),
            PhysicsLayers::BOUNDARY,
            PlayerBase::default(),
            Cleanup,
        ));

    // left boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(-ARENA_WIDTH * 0.5 - 16.0, 0.0, 0.0),
            sprite: Sprite {
                color: BOUNDARY_COLOR,
                custom_size: Some(Vec2::new(32.0, ARENA_HEIGHT + 64.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::new(Vec2::new(32.0, ARENA_HEIGHT + 64.0), 0.0, 1.0, 0.0),
            PhysicsLayers::BOUNDARY,
            BounceAudio::Bounce,
            Cleanup,
        ));

    // right boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(ARENA_WIDTH * 0.5 + 16.0, 0.0, 0.0),
            sprite: Sprite {
                color: BOUNDARY_COLOR,
                custom_size: Some(Vec2::new(32.0, ARENA_HEIGHT + 64.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::new(Vec2::new(32.0, ARENA_HEIGHT + 64.0), 0.0, 1.0, 0.0),
            PhysicsLayers::BOUNDARY,
            BounceAudio::Bounce,
            Cleanup,
        ));
}

fn make_ui(mut commands: Commands, materials: Res<Materials>, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(4.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(Cleanup)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    color: HEALTH_BAR_COLOR.into(),
                    ..Default::default()
                })
                .insert(HealthBar);
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    color: HEALTH_BAR_TRACKER_COLOR.into(),
                    ..Default::default()
                })
                .insert(HealthBarTracker::default());
        });

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(16.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: Color::NONE.into(),
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
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, -160.0, 0.0),
            sprite: Sprite {
                custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                color: PADDLE_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::new(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT), 3.0, 2.0, 1.0),
            Motion::default(),
            PhysicsLayers::PLAYER,
            BounceAudio::Bounce,
            Controller::default(),
            Player::default(),
            PlayerAssist::default(),
            Cleanup,
        ))
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
                color: PADDLE_COLOR,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::new(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT), 3.0, 1.0, 1.0),
            Motion::default(),
            PhysicsLayers::PLAYER,
            BounceAudio::Bounce,
            Controller::default(),
            Enemy::default(),
            Cleanup,
        ))
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

fn make_ball(mut commands: Commands, materials: Res<Materials>) {
    let alpha = 1.0 / BALL_GHOSTS_COUNT as f32;
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            texture: materials.ball.clone(),
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, alpha),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::new(Vec2::new(BALL_SIZE, BALL_SIZE), 1.0, 1.0, 0.5),
            PhysicsLayers::BALL,
            BounceAudio::Bounce,
            Ball::default(),
            Trajectory::default(),
            Cleanup,
        ))
        .with_children(|parent| {
            for _ in 0..BALL_GHOSTS_COUNT {
                parent.spawn_bundle(SpriteBundle {
                    texture: materials.ball.clone(),
                    sprite: Sprite {
                        color: Color::rgba(1.0, 1.0, 1.0, alpha),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
        });
}

#[allow(clippy::type_complexity)]
fn reset_ball(
    mut commands: Commands,
    mut player_miss_events: EventReader<PlayerMissEvent>,
    mut player_hit_events: EventReader<PlayerHitEvent>,
    mut query: Query<(Entity, &mut Transform), (With<Ball>, With<Motion>)>,
) {
    let mut closure = |ball| -> Option<()> {
        let (_, mut transform) = query.get_mut(ball).ok()?;
        transform.translation = Vec3::new(0.0, 0.0, -1.0);
        commands.entity(ball).remove::<Motion>();
        Some(())
    };

    for event in player_hit_events.iter() {
        if event.win {
            closure(event.ball);
        }
    }

    for event in player_miss_events.iter() {
        closure(event.ball);
    }

    // reset if the ball if out of range
    for (entity, mut transform) in query.iter_mut() {
        if transform.translation.x < -ARENA_WIDTH / 2.0
            || transform.translation.x > ARENA_WIDTH / 2.0
            || transform.translation.y < -ARENA_HEIGHT / 2.0
            || transform.translation.y > ARENA_HEIGHT / 2.0
        {
            transform.translation = Vec3::new(0.0, 0.0, -1.0);
            commands.entity(entity).remove::<Motion>();
        }
    }
}

fn remove_ball(
    mut commands: Commands,
    mut player_miss_events: EventReader<PlayerMissEvent>,
    mut player_hit_events: EventReader<PlayerHitEvent>,
    query: Query<(), With<Ball>>,
) {
    for event in player_miss_events.iter() {
        if event.lose && query.contains(event.ball) {
            commands.entity(event.ball).remove::<Ball>();
        }
    }

    for event in player_hit_events.iter() {
        if event.win && query.contains(event.ball) {
            commands.entity(event.ball).remove::<Ball>();
        }
    }
}

fn make_player_hint(
    mut commands: Commands,
    materials: Res<Materials>,
    query: Query<Entity, (Added<Player>, Without<Hint>)>,
) {
    for entity in query.iter() {
        let hint = commands
            .spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(0.0, ARENA_HEIGHT / 2.0, 0.0),
                texture: materials.hint.clone(),
                sprite: Sprite {
                    color: HINT_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Cleanup)
            .id();

        commands.entity(entity).insert(Hint(hint));
    }
}

fn make_ball_hint(
    mut commands: Commands,
    materials: Res<Materials>,
    query: Query<Entity, (Added<Ball>, Without<Hint>)>,
) {
    for entity in query.iter() {
        let hint = commands
            .spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0, 0.0),
                texture: materials.hint.clone(),
                sprite: Sprite {
                    color: HINT_COLOR,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Cleanup)
            .id();

        commands.entity(entity).insert(Hint(hint));
    }
}

/// Decreases enemy base's health when the ball hits it.
/// Sends [`PlayerHitEvent`] and [`GameOverEvent::Win`].
#[allow(clippy::too_many_arguments)]
fn player_hit(
    time: Res<Time>,
    mut timer: ResMut<Debounce>,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_hit_events: EventWriter<PlayerHitEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    ball_query: Query<(&RigidBody, &Motion), With<Ball>>,
    mut base_query: Query<&mut EnemyBase, Without<Ball>>,
) {
    if timer.hit.tick(time.delta()).finished() {
        for event in collision_events.iter() {
            let mut closure = |ball: Entity, base: Entity| -> Option<()> {
                let (rigid_body, motion) = ball_query.get(ball).ok()?;
                let mut base = base_query.get_mut(base).ok()?;

                let location = event.hit.location();
                let hp = base.hp;

                let mass = rigid_body.mass();
                let speed = motion.velocity.length();
                let damage = hp.min(speed * mass).min(MAX_DAMAGE);

                base.hp -= damage;

                let win = base.hp <= 0.0;
                if win {
                    game_over_events.send(GameOverEvent::Win);
                }
                timer.hit.reset();

                player_hit_events.send(PlayerHitEvent {
                    ball,
                    location,
                    win,
                });

                Some(())
            };

            closure(event.entities[0], event.entities[1])
                .or_else(|| closure(event.entities[1], event.entities[0]));
        }
    }
}

/// Decreases player's ball count when the ball hits player's base.
/// Sends [`PlayerMissEvent`] and [`GameOverEvent::Lose`].
#[allow(clippy::too_many_arguments)]
fn player_miss(
    time: Res<Time>,
    mut timer: ResMut<Debounce>,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_miss_events: EventWriter<PlayerMissEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    ball_query: Query<(), With<Ball>>,
    mut base_query: Query<&mut PlayerBase, Without<Ball>>,
) {
    if timer.miss.tick(time.delta()).finished() {
        for event in collision_events.iter() {
            let mut closure = |ball: Entity, base: Entity| -> Option<()> {
                ball_query.get(ball).ok()?;
                let mut base = base_query.get_mut(base).ok()?;

                let location = event.hit.location();

                let lose = base.ball_count == 0;
                if lose {
                    game_over_events.send(GameOverEvent::Lose);
                } else {
                    base.ball_count -= 1;
                }
                timer.miss.reset();

                player_miss_events.send(PlayerMissEvent {
                    ball,
                    location,
                    lose,
                });

                Some(())
            };

            closure(event.entities[0], event.entities[1])
                .or_else(|| closure(event.entities[1], event.entities[0]));
        }
    }
}

/// Emits [`BounceEvent`] when there the ball hits something after debouncing.
#[allow(clippy::too_many_arguments)]
fn ball_bounce(
    time: Res<Time>,
    mut timer: ResMut<Debounce>,
    mut collision_events: EventReader<CollisionEvent>,
    mut bounce_events: EventWriter<BounceEvent>,
    ball_query: Query<(), With<Ball>>,
) {
    if timer.bounce.tick(time.delta()).finished() {
        for event in collision_events.iter() {
            let mut closure = |ball: Entity, other: Entity| -> Option<()> {
                ball_query.get(ball).ok()?;

                let location = event.hit.location();
                bounce_events.send(BounceEvent {
                    ball,
                    other,
                    location,
                });

                timer.bounce.reset();
                Some(())
            };

            closure(event.entities[0], event.entities[1])
                .or_else(|| closure(event.entities[1], event.entities[0]));
        }
    }
}

/// Emits [`CameraShakeEvent`] and create hit blast effects when the ball hits something (with debouncing).
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn bounce_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<Debounce>,
    mut collision_events: EventReader<CollisionEvent>,
    mut camera_shake_events: EventWriter<CameraShakeEvent>,
    mut bounce_entities: Local<Option<[Entity; 2]>>,
    materials: Res<Materials>,
    query: Query<(), With<Ball>>,
    motions: Query<Option<&Motion>>,
) {
    if timer.effects.tick(time.delta()).finished() {
        if collision_events.is_empty() {
            *bounce_entities = None;
        }

        for event in collision_events.iter() {
            let results = event.entities.map(|entity| query.get(entity).is_ok());
            if results.contains(&true) {
                if bounce_entities.map_or(true, |entities| entities != event.entities) {
                    let velocities = motions.many(event.entities).map(|maybe_motion| {
                        maybe_motion.map_or(Vec2::ZERO, |motion| motion.velocity)
                    });

                    let velocity = if results[0] {
                        velocities[0] - velocities[1]
                    } else {
                        velocities[1] - velocities[0]
                    };

                    let speed = velocity.length();
                    let scale = (speed / MAX_BOUNCE_EFFECTS_SPEED).min(1.0);

                    // screen shake
                    let amplitude = velocity.normalize() * scale * 8.0;
                    camera_shake_events.send(CameraShakeEvent { amplitude });
                    timer.effects.reset();

                    // hit effect
                    commands
                        .spawn_bundle(SpriteSheetBundle {
                            transform: Transform {
                                translation: event.hit.location().extend(0.0),
                                rotation: Quat::from_rotation_z(
                                    f32::atan2(-velocity.y, -velocity.x) + FRAC_PI_4,
                                ),
                                scale: Vec3::new(0.2, 0.2, 1.0),
                            },
                            texture_atlas: materials.hit.clone(),
                            ..Default::default()
                        })
                        .insert(HitEffect::default())
                        .insert(Cleanup);
                }

                *bounce_entities = Some(event.entities);
            }
        }
    }
}

/// Creates full-screen explosion effects both when player lose or win.
fn score_effects(
    mut commands: Commands,
    materials: Res<Materials>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<DeathEffectMaterial>>,
    mut player_miss_events: EventReader<PlayerMissEvent>,
    mut player_hit_events: EventReader<PlayerHitEvent>,
) {
    let mut make_effect = |location: Vec2, duration: f32| {
        for offset in [
            Vec2::new(-100.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(0.0, -100.0),
            Vec2::new(0.0, 100.0),
        ] {
            commands
                .spawn_bundle(MaterialMesh2dBundle {
                    mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                    material: color_materials.add(materials.death.clone().into()),
                    transform: Transform::from_translation((location + offset).extend(0.9)),
                    ..Default::default()
                })
                .insert(DeathEffect {
                    timer: Timer::from_seconds(duration, false),
                    speed: DEATH_EFFECT_SPEED,
                    acceleration: DEATH_EFFECT_ACCELERATION,
                })
                .insert(Cleanup);
        }
    };

    for event in player_miss_events.iter() {
        let duration = if event.lose { 2.0 } else { 1.0 };
        make_effect(event.location, duration);
    }

    for event in player_hit_events.iter() {
        if event.win {
            let duration = 2.0;
            make_effect(event.location, duration);
        }
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

#[allow(clippy::too_many_arguments)]
fn bounce_audio(
    audios: Res<Audios>,
    mut audio: ResMut<DynamicAudioChannels>,
    volume: Res<AudioVolume>,
    time: Res<Time>,
    mut timer: ResMut<Debounce>,
    mut events: EventReader<CollisionEvent>,
    mut index: Local<usize>,
    mut bounce_entities: Local<Option<[Entity; 2]>>,
    query: Query<(Entity, &BounceAudio)>,
    balls: Query<(), With<Ball>>,
    motions: Query<Option<&Motion>>,
) {
    let mut can_play_audio = timer.audio_bounce_long.tick(time.delta()).finished();
    timer.audio_bounce_short.tick(time.delta());
    timer.audio_hit.tick(time.delta());

    let channels = (0..AUDIO_CHANNEL_COUNT)
        .map(|index| format!("impact_{}", index))
        .collect_vec();

    for event in events.iter() {
        // one of the entities must be a ball
        let results = event.entities.map(|entity| balls.get(entity).is_ok());
        if !results.contains(&true) {
            continue;
        }

        let (entities, bounce_audio) = if let Ok(x) = query.get_many(event.entities) {
            let (entities, bounce_audios): (Vec<_>, Vec<_>) = x.iter().cloned().unzip();
            let bounce_audio = if bounce_audios.contains(&BounceAudio::Hit) {
                BounceAudio::Hit
            } else {
                BounceAudio::Bounce
            };
            (entities.try_into().ok(), bounce_audio)
        } else {
            continue;
        };

        let (audio_source, debounce_timer) = match bounce_audio {
            BounceAudio::Bounce => {
                let index = fastrand::usize(..IMPACT_AUDIOS.len());
                (
                    audios.impact_audios[index].clone(),
                    &timer.audio_bounce_short,
                )
            }
            BounceAudio::Hit => (audios.hit_audio.clone(), &timer.audio_hit),
        };

        if entities != *bounce_entities {
            can_play_audio = debounce_timer.finished();
            *bounce_entities = entities;
        }

        *index = (*index + 1) % AUDIO_CHANNEL_COUNT;
        let channel = audio.create_channel(&channels[*index]);

        if can_play_audio {
            let velocities = motions
                .many(event.entities)
                .map(|maybe_motion| maybe_motion.map_or(Vec2::ZERO, |motion| motion.velocity));
            let speed = (velocities[0] - velocities[1]).length();
            if speed > MIN_BOUNCE_AUDIO_SPEED {
                let normalized_speed = speed
                    .intermediate(MIN_BOUNCE_AUDIO_SPEED, MAX_BOUNCE_AUDIO_SPEED)
                    .clamp(0.0, 1.0);

                let panning = event.hit.location().x / ARENA_WIDTH + 0.5;
                channel.set_panning(panning.into());

                let volume = volume.effects * (0.5 * normalized_speed + 0.5);
                channel.set_volume(volume.into());

                let playback_rate = 0.4 * fastrand::f32() + 0.8;
                channel.set_playback_rate(playback_rate.into());

                channel.play(audio_source);

                timer.audio_bounce_long.reset();
                timer.audio_bounce_short.reset();
            }
        }
    }
}

fn score_audio(
    audios: Res<Audios>,
    audio: Res<AudioChannel<ScoreAudioChannel>>,
    volume: Res<AudioVolume>,
    mut player_miss_events: EventReader<PlayerMissEvent>,
    mut game_over_events: EventReader<GameOverEvent>,
) {
    for event in player_miss_events.iter() {
        // let channel = &AudioChannel::new("miss".into());
        let panning = event.location.x / ARENA_WIDTH + 0.5;
        // audio.set_volume_in_channel(volume.effects, channel);
        // audio.set_panning_in_channel(panning, channel);
        // audio.play_in_channel(audios.miss_audio.clone(), channel);
        audio.set_volume(volume.effects.into());
        audio.set_panning(panning.into());
        audio.play(audios.miss_audio.clone());
    }

    for event in game_over_events.iter() {
        // let channel = &AudioChannel::new("over".into());
        // audio.set_volume_in_channel(volume.effects, channel);
        audio.set_volume(volume.effects.into());
        match event {
            GameOverEvent::Win => audio.play(audios.explosion_audio.clone()),
            GameOverEvent::Lose => audio.play(audios.lose_audio.clone()),
        };
    }
}
