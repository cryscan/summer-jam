use bevy::prelude::*;
use bevy_kira_audio::AudioPlugin;
use wasm_bindgen::prelude::*;

mod background;
mod config;
mod game;
mod loading;
mod menu;
mod score;
mod utils;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Loading,
    Menu,
    Settings,
    Game,
    Tutorial,
    Win,
}

#[derive(Reflect)]
pub struct TimeScale(pub f32);

impl Default for TimeScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl TimeScale {
    pub fn reset(&mut self) {
        self.0 = 1.0;
    }
}

#[derive(Reflect)]
pub struct AudioVolume {
    pub music: f32,
    pub effects: f32,
}

pub struct MusicTrack(&'static str);

#[wasm_bindgen]
pub fn run() {
    let mut app = App::new();

    app.register_type::<TimeScale>()
        .register_type::<AudioVolume>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(WindowDescriptor {
            title: "Bounce Up!".into(),
            width: config::ARENA_WIDTH,
            height: config::ARENA_HEIGHT,
            resizable: false,
            ..Default::default()
        })
        .init_resource::<TimeScale>()
        .insert_resource(AudioVolume {
            music: 0.3,
            effects: 1.0,
        })
        .insert_resource(MusicTrack(""));

    #[cfg(feature = "dot")]
    app.add_plugins_with(DefaultPlugins, |plugins| {
        plugins.disable::<bevy::log::LogPlugin>()
    });

    #[cfg(not(feature = "dot"))]
    app.add_plugins(DefaultPlugins);

    app.add_plugin(AudioPlugin)
        .add_state(AppState::Loading)
        .add_startup_system(setup)
        .add_system(lock_release_cursor)
        .add_plugin(loading::LoadingPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(score::ScorePlugin)
        .add_plugin(background::BackgroundPlugin);

    #[cfg(feature = "dot")]
    bevy_mod_debugdump::print_render_schedule(&mut app);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

fn lock_release_cursor(app_state: Res<State<AppState>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        match app_state.current() {
            AppState::Game | AppState::Tutorial => {
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
