#[macro_use]
extern crate derive_new;

use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin};
use config::BACKGROUND_MUSIC;
use wasm_bindgen::prelude::*;

mod background;
mod config;
mod game;
mod loading;
mod score;
mod title;
mod utils;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Loading,
    Title,
    Game,
    Win,
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(WindowDescriptor {
            title: "Bounce Up!".into(),
            width: config::ARENA_WIDTH,
            height: config::ARENA_HEIGHT,
            resizable: false,
            ..Default::default()
        })
        .init_resource::<utils::TimeScale>();

    #[cfg(feature = "dot")]
    app.add_plugins_with(DefaultPlugins, |plugins| {
        plugins.disable::<bevy::log::LogPlugin>()
    });

    #[cfg(not(feature = "dot"))]
    app.add_plugins(DefaultPlugins);

    app.add_plugin(AudioPlugin)
        // .add_plugin(HanabiPlugin)
        .add_state(AppState::Loading)
        .add_startup_system(setup)
        .add_system(lock_release_cursor)
        .add_plugin(loading::LoadingPlugin)
        .add_plugin(title::TitlePlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(score::ScorePlugin)
        .add_plugin(background::BackgroundPlugin);

    #[cfg(feature = "dot")]
    bevy_mod_debugdump::print_render_schedule(&mut app);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let channel = AudioChannel::new("background".into());
    audio.play_looped_in_channel(asset_server.load(BACKGROUND_MUSIC), &channel);
    // audio.set_playback_rate_in_channel(1.5, &channel);
    audio.set_volume_in_channel(0.5, &channel);
}

fn lock_release_cursor(app_state: Res<State<AppState>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        match app_state.current() {
            AppState::Game => {
                window.set_cursor_lock_mode(true);
                window.set_cursor_visibility(false);
            }
            _ => {
                window.set_cursor_lock_mode(false);
                window.set_cursor_visibility(true);
            }
        }
    }
}
