use self::{ball::*, base::*, effects::*, enemy::*, hint::*, physics::*, player::*};
use crate::{
    config::*,
    score::Score,
    utils::{cleanup_system, Damp, Interpolation, MusicVolume, TimeScale},
    AppState,
};
use bevy::{core::FixedTimestep, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_kira_audio::{Audio, AudioChannel, AudioSource};
use itertools::Itertools;

mod ball;
mod base;
mod effects;
mod enemy;
mod hint;
mod physics;
mod player;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameOverEvent>()
            .add_event::<MakeBallEvent>()
            .add_event::<PlayerHitEvent>()
            .add_event::<PlayerMissEvent>()
            .insert_resource(Debounce {
                bounce_long: Timer::from_seconds(0.5, false),
                bounce_short: Timer::from_seconds(0.1, false),
                effects: Timer::from_seconds(0.1, false),
                hit: Timer::from_seconds(0.1, false),
                miss: Timer::from_seconds(0.5, false),
            })
            .insert_resource(GameOver {
                slow_motion_timer: Timer::from_seconds(0.8, false),
                state_change_timer: Timer::from_seconds(2.0, false),
                event: None,
            })
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
                    .with_system(destroy_ball.after(make_ball))
                    .with_system(move_player)
                    .with_system(move_enemy)
                    .with_system(move_ball)
                    .with_system(activate_ball)
                    .with_system(update_ball)
                    .with_system(assist_player)
                    .with_system(player_hit)
                    .with_system(player_miss)
                    .with_system(count_ball)
                    .with_system(health_bar)
                    .with_system(health_bar_tracker)
                    .with_system(hint_system)
                    .with_system(score_system)
                    .with_system(score_effects)
                    .with_system(bounce_effects)
                    .with_system(game_over),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Game).with_system(cleanup_system::<Cleanup>),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(AI_TIME_STEP))
                    .with_system(predict_ball)
                    .with_system(control_enemy),
            )
            .add_system_set(
                SystemSet::new()
                    .with_system(bounce_audio)
                    .with_system(score_audio),
            )
            .add_plugin(PhysicsPlugin)
            .add_plugin(EffectsPlugin);
    }
}

#[derive(Clone, Copy)]
enum GameOverEvent {
    Win,
    Lose,
}

struct MakeBallEvent;

struct PlayerHitEvent {
    ball_entity: Entity,
    hp: f32,
    damage: f32,
    location: Vec2,
}

struct PlayerMissEvent {
    ball_entity: Entity,
    ball_count: i32,
    location: Vec2,
}

struct Debounce {
    bounce_long: Timer,
    bounce_short: Timer,
    effects: Timer,
    hit: Timer,
    miss: Timer,
}

struct GameOver {
    slow_motion_timer: Timer,
    state_change_timer: Timer,
    event: Option<GameOverEvent>,
}

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
    death: Handle<Image>,

    // static entities
    boundary: Color,
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
        paddle: Color::rgba_u8(155, 173, 183, 100),
        ball: asset_server.load(BALL_SPRITE),
        hint: asset_server.load(HINT_SPRITE),
        death: asset_server.load(DEATH_SPRITE),

        boundary: Color::NONE,
        separate: Color::rgba(0.5, 0.5, 0.5, 0.2),

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

#[allow(clippy::too_many_arguments)]
fn enter_game(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    volume: Res<MusicVolume>,
    time: Res<Time>,
    mut time_scale: ResMut<TimeScale>,
    mut score: ResMut<Score>,
    mut game_over: ResMut<GameOver>,
    mut make_ball_events: EventWriter<MakeBallEvent>,
) {
    info!("Entering Game");

    // clear score state
    score.timestamp = time.seconds_since_startup();
    score.hits = 0;
    score.miss = 0;

    time_scale.reset();

    game_over.slow_motion_timer.reset();
    game_over.state_change_timer.reset();
    game_over.event = None;

    make_ball_events.send(MakeBallEvent);

    audio.stop();
    audio.set_volume(volume.0);
    audio.set_playback_rate(1.2);
    audio.play_looped(asset_server.load(GAME_MUSIC));
}

fn update_game(mut app_state: ResMut<State<AppState>>, mut input: ResMut<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        input.reset(KeyCode::Escape);
        app_state.set(AppState::Title).unwrap();
    }
}

fn make_static_entities(mut commands: Commands, materials: Res<Materials>) {
    // middle Separate
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, 8.0, 0.0),
            sprite: Sprite {
                color: materials.boundary,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 32.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(RigidBody::new(Vec2::new(ARENA_WIDTH, 32.0), 0.0, 0.9, 0.5))
        .insert(PhysicsLayers::SEPARATE);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: materials.separate,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 16.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup);

    // top boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, ARENA_HEIGHT * 0.5 + 16.0, 0.0),
            sprite: Sprite {
                color: materials.boundary,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 32.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(EnemyBase {
            full_hp: ENEMY_BASE_FULL_HP,
            hp: ENEMY_BASE_FULL_HP,
        })
        .insert(RigidBody::new(Vec2::new(ARENA_WIDTH, 32.0), 0.0, 0.9, 0.0))
        .insert(PhysicsLayers::BOUNDARY);

    // bottom boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(0.0, -ARENA_HEIGHT * 0.5 - 16.0, 0.0),
            sprite: Sprite {
                color: materials.boundary,
                custom_size: Some(Vec2::new(ARENA_WIDTH, 32.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(PlayerBase { ball_count: 3 })
        .insert(RigidBody::new(Vec2::new(ARENA_WIDTH, 32.0), 0.0, 0.9, 0.5))
        .insert(PhysicsLayers::BOUNDARY);

    // left boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(-ARENA_WIDTH * 0.5 - 16.0, 0.0, 0.0),
            sprite: Sprite {
                color: materials.boundary,
                custom_size: Some(Vec2::new(32.0, ARENA_HEIGHT + 64.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(RigidBody::new(
            Vec2::new(32.0, ARENA_HEIGHT + 64.0),
            0.0,
            1.0,
            0.0,
        ))
        .insert(PhysicsLayers::BOUNDARY);

    // right boundary
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_xyz(ARENA_WIDTH * 0.5 + 16.0, 0.0, 0.0),
            sprite: Sprite {
                color: materials.boundary,
                custom_size: Some(Vec2::new(32.0, ARENA_HEIGHT + 64.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cleanup)
        .insert(RigidBody::new(
            Vec2::new(32.0, ARENA_HEIGHT + 64.0),
            0.0,
            1.0,
            0.0,
        ))
        .insert(PhysicsLayers::BOUNDARY);
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
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                ..Default::default()
            },
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
        .insert(Player {
            max_speed: PLAYER_MAX_SPEED,
            sensitivity: PLAYER_SENSITIVITY,
            damp: PLAYER_DAMP,
        })
        .insert(PlayerAssist {
            range: PLAYER_ASSIST_RANGE,
            speed: PLAYER_ASSIST_SPEED,
            vertical_speed_threshold: PLAYER_ASSIST_VERTICAL_SPEED_THRESHOLD,
            speed_threshold: PLAYER_ASSIST_SPEED_THRESHOLD,
        })
        .insert(Controller::new())
        .insert(RigidBody::new(
            Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
            3.0,
            2.0,
            1.0,
        ))
        .insert(PhysicsLayers::PLAYER)
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
        .insert(Enemy {
            min_speed: ENEMY_MIN_SPEED,
            max_speed: ENEMY_MAX_SPEED,
            normal_speed: ENEMY_NORMAL_SPEED,
            damp: ENEMY_DAMP,
            hit_range: Vec2::new(ENEMY_HIT_RANGE_HORIZONTAL, ENEMY_HIT_RANGE_VERTICAL),
            hit_speed_threshold: ENEMY_HIT_SPEED_THRESHOLD,
            hit_height_threshold: 0.125 * ARENA_HEIGHT,
        })
        .insert(Controller::new())
        .insert(RigidBody::new(
            Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
            3.0,
            1.0,
            1.0,
        ))
        .insert(PhysicsLayers::PLAYER)
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

fn make_ball(
    mut commands: Commands,
    materials: Res<Materials>,
    mut events: EventReader<MakeBallEvent>,
) {
    for _ in events.iter() {
        let hint = commands
            .spawn_bundle(SpriteBundle {
                transform: Transform::from_xyz(0.0, -ARENA_HEIGHT / 2.0, 0.0),
                texture: materials.hint.clone(),
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Cleanup)
            .id();

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
            .insert(Cleanup)
            .insert(Ball::default())
            .insert(RigidBody::new(
                Vec2::new(BALL_SIZE, BALL_SIZE),
                1.0,
                1.0,
                0.5,
            ))
            .insert(PhysicsLayers::BALL)
            .insert(Trajectory {
                start_time: 0.0,
                points: vec![Point::default(); PREDICT_SIZE],
            })
            .insert(Hint(hint))
            .insert(BounceAudio)
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
}

fn destroy_ball(
    mut commands: Commands,
    mut player_miss_events: EventReader<PlayerMissEvent>,
    mut player_hit_events: EventReader<PlayerHitEvent>,
    ball_query: Query<Option<&Hint>, With<Ball>>,
) {
    let mut closure = |ball_entity| -> Option<()> {
        if let Some(hint) = ball_query.get(ball_entity).ok()? {
            commands.entity(hint.0).despawn();
        }
        commands.entity(ball_entity).despawn_recursive();
        Some(())
    };

    for event in player_hit_events.iter() {
        if event.hp <= event.damage {
            closure(event.ball_entity);
        }
    }

    for event in player_miss_events.iter() {
        closure(event.ball_entity);
    }
}

#[allow(clippy::too_many_arguments)]
fn player_hit(
    time: Res<Time>,
    mut timer: ResMut<Debounce>,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_hit_events: EventWriter<PlayerHitEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    ball_query: Query<(&RigidBody, &Motion), With<Ball>>,
    mut base_query: Query<&mut EnemyBase>,
) {
    let can_hit = timer.hit.tick(time.delta()).finished();

    let mut closure = |ball_entity: Entity, base_entity: Entity| -> Option<()> {
        let (rigid_body, motion) = ball_query.get(ball_entity).ok()?;
        let mass = rigid_body.mass();
        let speed = motion.velocity.length();

        let mut base = base_query.get_mut(base_entity).ok()?;
        let hp = base.hp;
        let damage = hp.min(speed * mass).min(MAX_DAMAGE);

        player_hit_events.send(PlayerHitEvent {
            ball_entity,
            hp,
            damage,
            location: motion.translation.truncate(),
        });

        base.hp -= damage;
        if base.hp <= 0.0 {
            game_over_events.send(GameOverEvent::Win);
        }
        timer.hit.reset();

        Some(())
    };

    if can_hit {
        for event in collision_events.iter() {
            closure(event.entities[0], event.entities[1]);
            closure(event.entities[1], event.entities[0]);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn player_miss(
    time: Res<Time>,
    mut timer: ResMut<Debounce>,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_miss_events: EventWriter<PlayerMissEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
    mut make_ball_events: EventWriter<MakeBallEvent>,
    ball_query: Query<&Motion, With<Ball>>,
    mut base_query: Query<(Entity, &mut PlayerBase), Without<Ball>>,
) {
    let can_miss = timer.miss.tick(time.delta()).finished();
    let (entity, mut base) = base_query.single_mut();

    let mut closure = |ball_entity: Entity, base_entity: Entity| -> Option<()> {
        if base_entity != entity {
            return None;
        }

        let motion = ball_query.get(ball_entity).ok()?;
        let ball_count = base.ball_count;

        player_miss_events.send(PlayerMissEvent {
            ball_entity,
            ball_count,
            location: motion.translation.truncate(),
        });

        if base.ball_count == 0 {
            game_over_events.send(GameOverEvent::Lose);
        } else {
            base.ball_count -= 1;
            make_ball_events.send(MakeBallEvent);
        }
        timer.miss.reset();

        Some(())
    };

    if can_miss {
        for event in collision_events.iter() {
            closure(event.entities[0], event.entities[1]);
            closure(event.entities[1], event.entities[0]);
        }
    }
}

fn game_over(
    time: Res<Time>,
    mut time_scale: ResMut<TimeScale>,
    mut app_state: ResMut<State<AppState>>,
    mut game_over_events: EventReader<GameOverEvent>,
    mut game_over: ResMut<GameOver>,
) {
    if let Some(event) = game_over.event {
        let mut target_time_scale = 0.2;
        let mut time_scale_damp = TIME_SCALE_DAMP;

        if game_over.slow_motion_timer.tick(time.delta()).finished() {
            target_time_scale = 1.0;
            time_scale_damp = GAME_OVER_TIME_SCALE_DAMP;
        }
        time_scale.0 = time_scale
            .0
            .damp(target_time_scale, time_scale_damp, time.delta_seconds());

        // it's time to switch state
        if game_over
            .state_change_timer
            .tick(time.delta())
            .just_finished()
        {
            time_scale.reset();
            match event {
                GameOverEvent::Win => app_state.set(AppState::Win).unwrap(),
                GameOverEvent::Lose => app_state.set(AppState::Title).unwrap(),
            }
        }
    } else {
        for event in game_over_events.iter() {
            game_over.event = Some(*event);
            time_scale.0 = 0.2;
        }
    }
}

#[allow(clippy::type_complexity)]
fn bounce_effects(
    time: Res<Time>,
    mut timer: ResMut<Debounce>,
    mut collision_events: EventReader<CollisionEvent>,
    mut camera_shake_events: EventWriter<CameraShakeEvent>,
    mut bounce_entities: Local<Option<[Entity; 2]>>,
    query: Query<Entity, With<Ball>>,
) {
    if timer.effects.tick(time.delta()).finished() {
        if collision_events.is_empty() {
            *bounce_entities = None;
        }

        for event in collision_events.iter() {
            if query
                .get(event.entities[0])
                .or_else(|_| query.get(event.entities[1]))
                .is_ok()
            {
                if bounce_entities.map_or(true, |entities| entities != event.entities) {
                    let speed = event.velocity.length();
                    let scale = (speed / MAX_BOUNCE_EFFECTS_SPEED).min(1.0);

                    // screen shake
                    let amplitude = event.velocity.normalize() * scale * 8.0;
                    camera_shake_events.send(CameraShakeEvent { amplitude });
                    timer.effects.reset();
                }

                *bounce_entities = Some(event.entities);
            }
        }
    }
}

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
                    transform: Transform::from_translation((location + offset).extend(0.01)),
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
        let duration = if event.ball_count <= 0 { 2.0 } else { 1.0 };
        make_effect(event.location, duration);
    }

    for event in player_hit_events.iter() {
        if event.hp - event.damage <= 0.0 {
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
    audio: Res<Audio>,
    time: Res<Time>,
    // time_scale: Res<TimeScale>,
    mut timer: ResMut<Debounce>,
    mut events: EventReader<CollisionEvent>,
    mut index: Local<usize>,
    mut bounce_entities: Local<Option<[Entity; 2]>>,
    query: Query<Entity, With<BounceAudio>>,
) {
    let mut can_play_audio = timer.bounce_long.tick(time.delta()).finished();
    timer.bounce_short.tick(time.delta());

    let channels = (0..MAX_IMPACT_AUDIO_CHANNELS)
        .map(|index| AudioChannel::new(format!("impact_{}", index)))
        .collect_vec();

    // for channel in channels.iter() {
    //     audio.set_playback_rate_in_channel(time_scale.0, channel);
    // }

    for event in events.iter() {
        let entities = query.get_many(event.entities).ok();
        if entities.is_none() {
            continue;
        }

        // bounce happens between a different pair
        if entities != *bounce_entities {
            can_play_audio = timer.bounce_short.finished();
            *bounce_entities = entities;
        }

        *index = (*index + 1) % MAX_IMPACT_AUDIO_CHANNELS;
        let channel = &channels[*index];

        if can_play_audio {
            let speed = event.velocity.length();
            if speed > MIN_BOUNCE_AUDIO_SPEED {
                let normalized_speed = speed
                    .intermediate(MIN_BOUNCE_AUDIO_SPEED, MAX_BOUNCE_AUDIO_SPEED)
                    .clamp(0.0, 1.0);

                let panning = event.hit.location().x / ARENA_WIDTH + 0.5;
                audio.set_panning_in_channel(panning, channel);

                let volume = 0.2 * normalized_speed + 0.05;
                audio.set_volume_in_channel(volume, channel);

                let pitch = ((normalized_speed * 4.0) as usize).min(3);
                let audio_source = audios.impact_audios[pitch].clone();
                audio.play_in_channel(audio_source, channel);

                timer.bounce_long.reset();
                timer.bounce_short.reset();
            }
        }
    }
}

fn score_audio(
    audios: Res<Audios>,
    audio: Res<Audio>,
    // time_scale: Res<TimeScale>,
    mut player_hit_events: EventReader<PlayerHitEvent>,
    mut player_miss_events: EventReader<PlayerMissEvent>,
    mut game_over_events: EventReader<GameOverEvent>,
) {
    let channel = &AudioChannel::new("score".into());
    // audio.set_playback_rate_in_channel(time_scale.0, channel);

    for _ in player_hit_events.iter() {
        audio.play_in_channel(audios.hit_audio.clone(), channel);
    }

    for _ in player_miss_events.iter() {
        audio.play_in_channel(audios.miss_audio.clone(), channel);
    }

    for event in game_over_events.iter() {
        match event {
            GameOverEvent::Win => audio.play_in_channel(audios.explosion_audio.clone(), channel),
            GameOverEvent::Lose => audio.play_in_channel(audios.lose_audio.clone(), channel),
        };
    }
}
