use super::*;

pub struct PracticePlugin;

impl Plugin for PracticePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(PracticeState::Plain)
            .add_system_set(
                SystemSet::on_enter(AppState::Practice)
                    .with_system(enter_practice)
                    .with_system(make_arena)
                    .with_system(make_ui)
                    .with_system(make_player)
                    .with_system(make_ball),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Practice)
                    .with_system(escape_system)
                    .with_system(progress_system)
                    .with_system(change_slits)
                    .with_system(validate_slit_block)
                    .with_system(reset_ball)
                    .with_system(player_hit)
                    .with_system(player_miss)
                    .with_system(recover_enemy_health)
                    .with_system(player_ball_infinite),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Practice).with_system(cleanup_system::<Cleanup>),
            )
            .add_system_set(
                SystemSet::on_enter(PracticeState::Slits).with_system(make_slit_blocks),
            );
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PracticeState {
    Plain,
    Slits,
}

#[allow(clippy::too_many_arguments)]
fn enter_practice(
    mut practice_state: ResMut<State<PracticeState>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    volume: Res<AudioVolume>,
    mut music_track: ResMut<MusicTrack>,
    mut time_scale: ResMut<TimeScale>,
    mut heal_events: EventWriter<HealEvent>,
) {
    let _ = practice_state.set(PracticeState::Plain);

    time_scale.reset();

    heal_events.send(HealEvent(Heal::default()));

    if music_track.0 != GAME_MUSIC {
        audio.stop();
        audio.set_volume(volume.music.into());
        audio.set_playback_rate(1.2);
        audio.play(asset_server.load(GAME_MUSIC)).looped();

        music_track.0 = GAME_MUSIC;
    }
}

/// Triggers a full recovery of enemy base health after beating it.
fn recover_enemy_health(
    time: Res<Time>,
    mut game_over_events: EventReader<GameOverEvent>,
    mut game_over: Local<GameOver>,
    mut heal_events: EventWriter<HealEvent>,
) {
    if let Some(event) = game_over.event {
        // it's time to switch state
        if game_over
            .state_change_timer
            .tick(time.delta())
            .just_finished()
        {
            *game_over = GameOver::default();

            match event {
                GameOverEvent::Win => heal_events.send(HealEvent(Heal::default())),
                GameOverEvent::Lose => {}
            }
        }
    } else {
        for event in game_over_events.iter() {
            game_over.event = Some(*event);
        }
    }
}

/// Make the player's ball count infinite.
fn player_ball_infinite(mut query: Query<&mut PlayerBase>) {
    if let Ok(mut base) = query.get_single_mut() {
        base.ball_count = 99;
    }
}

fn progress_system(
    time: Res<Time>,
    mut practice_state: ResMut<State<PracticeState>>,
    mut game_over_events: EventReader<GameOverEvent>,
    mut game_over: Local<GameOver>,
) {
    if game_over.event.is_some() {
        // it's time to switch state
        if game_over
            .state_change_timer
            .tick(time.delta())
            .just_finished()
        {
            *game_over = GameOver::default();
            let _ = practice_state.set(PracticeState::Slits);
        }
    } else {
        for event in game_over_events.iter() {
            game_over.event = Some(*event);
        }
    }
}

fn make_slit_blocks(mut commands: Commands, materials: Res<Materials>, mut slits: ResMut<Slits>) {
    let slits_index = slits.count / 2;
    slits.state = SlitState::Stand(slits_index);

    for index in 0..slits.count {
        let slit_block = SlitBlock {
            width: SLIT_BLOCK_WIDTH,
            index,
        };

        commands
            .spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(
                        slit_block.position(slits_index),
                        SLIT_POSITION_VERTICAL,
                        0.1,
                    ),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(SLIT_BLOCK_WIDTH, SLIT_BLOCK_HEIGHT)),
                        color: PADDLE_COLOR,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                RigidBody::new(
                    Vec2::new(SLIT_BLOCK_WIDTH, SLIT_BLOCK_HEIGHT),
                    0.0,
                    1.0,
                    0.0,
                ),
                PhysicsLayers::BOUNDARY,
                BounceAudio::Bounce,
                slit_block,
                Cleanup,
            ))
            .with_children(|parent| {
                parent.spawn(SpriteBundle {
                    transform: Transform::from_xyz(-PADDLE_WIDTH / 2.0 + 8.0, 0.0, 0.1),
                    texture: materials.enemy.clone(),
                    ..Default::default()
                });

                parent.spawn(SpriteBundle {
                    transform: Transform::from_xyz(PADDLE_WIDTH / 2.0 - 8.0, 0.0, 0.1),
                    texture: materials.enemy.clone(),
                    ..Default::default()
                });
            });
    }
}

fn change_slits(mut slits: ResMut<Slits>, mut player_hit_events: EventReader<PlayerHitEvent>) {
    for _ in player_hit_events.iter() {
        let previous = match &slits.state {
            SlitState::Stand(index) => *index,
            SlitState::Move { .. } => continue,
        };

        let mut next = fastrand::usize(0..=slits.count);
        if next == previous {
            next = (previous + 1) % (slits.count + 1);
        }
        slits.state = SlitState::Move {
            previous,
            next,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        };
    }
}

/// Temporary disables collision between ball and slit blocks when the ball is on top and moving down.
#[allow(clippy::type_complexity)]
fn validate_slit_block(
    mut query: Query<(&Transform, &mut PhysicsLayers, &mut Sprite), With<SlitBlock>>,
    balls: Query<(&Transform, &Motion), (With<Ball>, Without<SlitBlock>)>,
) {
    for (slit_block_transform, mut physics_layers, mut sprite) in query.iter_mut() {
        *physics_layers = PhysicsLayers::BOUNDARY;
        sprite.color = PADDLE_COLOR;

        for (ball_transform, motion) in balls.iter() {
            if ball_transform.translation.y + BALL_SIZE > slit_block_transform.translation.y
                && motion.velocity.y < 0.0
            {
                *physics_layers = PhysicsLayers::SEPARATE;
                sprite.color.set_a(0.2);
            }
        }
    }
}
