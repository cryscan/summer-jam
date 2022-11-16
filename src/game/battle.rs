use super::*;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Battle)
                .with_system(enter_battle)
                .with_system(make_arena)
                .with_system(make_ui)
                .with_system(make_player)
                .with_system(make_enemy)
                .with_system(make_ball),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Battle)
                // logical game-play systems
                .with_system(escape_system)
                .with_system(reset_ball)
                .with_system(remove_ball)
                .with_system(player_hit)
                .with_system(player_miss)
                .with_system(game_over_system),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::Battle).with_system(cleanup_system::<Cleanup>),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn enter_battle(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    volume: Res<AudioVolume>,
    mut music_track: ResMut<MusicTrack>,
    time: Res<Time>,
    mut time_scale: ResMut<TimeScale>,
    mut score: ResMut<Score>,
    mut heal_events: EventWriter<HealEvent>,
) {
    // clear score state
    score.timestamp = time.elapsed_seconds();
    score.hits = 0;
    score.miss = 0;

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

/// Deals with [`GameOverEvent`].
/// The system triggers a slow motion with the duration of [`GAME_OVER_SLOW_MOTION_DURATION`]
/// and also changes the [`AppState`] after [`GAME_OVER_STATE_CHANGE_DURATION`].
fn game_over_system(
    time: Res<Time>,
    mut time_scale: ResMut<TimeScale>,
    mut app_state: ResMut<State<AppState>>,
    mut game_over_events: EventReader<GameOverEvent>,
    mut game_over: Local<GameOver>,
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
            *game_over = GameOver::default();

            match event {
                GameOverEvent::Win => app_state.set(AppState::Win).unwrap(),
                GameOverEvent::Lose => app_state.set(AppState::Menu).unwrap(),
            }
        }
    } else {
        for event in game_over_events.iter() {
            game_over.event = Some(*event);
            time_scale.0 = 0.2;
        }
    }
}
