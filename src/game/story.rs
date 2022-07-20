use super::*;

pub struct StoryPlugin;

impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Story)
                .with_system(enter_story)
                .with_system(make_arena)
                .with_system(make_ui)
                .with_system(make_player)
                .with_system(make_enemy),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Story)
                // logical game-play systems
                .with_system(escape_system)
                .with_system(make_ball)
                .with_system(destroy_remake_ball)
                .with_system(player_hit)
                .with_system(player_miss)
                .with_system(game_over_system),
        )
        .add_system_set(SystemSet::on_exit(AppState::Story).with_system(cleanup_system::<Cleanup>));
    }
}

#[allow(clippy::too_many_arguments)]
fn enter_story(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    volume: Res<AudioVolume>,
    mut music_track: ResMut<MusicTrack>,
    time: Res<Time>,
    mut time_scale: ResMut<TimeScale>,
    mut score: ResMut<Score>,
    mut make_ball_events: EventWriter<MakeBallEvent>,
    mut heal_events: EventWriter<HealEvent>,
) {
    // clear score state
    score.timestamp = time.seconds_since_startup();
    score.hits = 0;
    score.miss = 0;

    time_scale.reset();

    make_ball_events.send(MakeBallEvent);
    heal_events.send(HealEvent(Heal::default()));

    if music_track.0 != GAME_MUSIC {
        audio.stop();
        audio.set_volume(volume.music);
        audio.set_playback_rate(1.2);
        audio.play_looped(asset_server.load(GAME_MUSIC));

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
