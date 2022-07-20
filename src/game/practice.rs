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
                    .with_system(make_player),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Practice)
                    .with_system(escape_system)
                    .with_system(progress_system)
                    .with_system(dynamic_slits)
                    .with_system(make_ball)
                    .with_system(destroy_remake_ball)
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
    mut make_ball_events: EventWriter<MakeBallEvent>,
    mut heal_events: EventWriter<HealEvent>,
) {
    let _ = practice_state.set(PracticeState::Plain);

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

/// Triggers a full recovery of enemy base health after beating it.
fn recover_enemy_health(
    mut game_over_events: EventReader<GameOverEvent>,
    mut heal_events: EventWriter<HealEvent>,
) {
    for event in game_over_events.iter() {
        match event {
            GameOverEvent::Win => heal_events.send(HealEvent(Heal::default())),
            GameOverEvent::Lose => {}
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
    mut practice_state: ResMut<State<PracticeState>>,
    game_over_events: EventReader<GameOverEvent>,
) {
    if !game_over_events.is_empty() {
        let _ = practice_state.set(PracticeState::Slits);
    }
}

fn dynamic_slits(mut slits: ResMut<Slits>, mut player_hit_events: EventReader<PlayerHitEvent>) {
    for _ in player_hit_events.iter() {
        let mut next_index = fastrand::usize(0..slits.count);
        if next_index == slits.index {
            next_index = (slits.index + 1) % slits.count;
        }
        slits.index = next_index;
    }
}
