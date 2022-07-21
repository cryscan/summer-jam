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
    Battle,
    Practice,
    Win,
}

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

pub struct AudioVolume {
    pub music: f32,
    pub effects: f32,
}

pub struct MusicTrack(&'static str);

#[derive(Component)]
pub struct TextColor {
    timer: Timer,
    colors: Vec<Color>,
    index: usize,
}

impl TextColor {
    pub fn new(colors: Vec<Color>, duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, true),
            colors,
            index: 0,
        }
    }
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
        .add_system(text_color_system)
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
            AppState::Battle | AppState::Practice => {
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

fn text_color_system(time: Res<Time>, mut query: Query<(&mut Text, &mut TextColor)>) {
    for (mut text, mut text_color) in query.iter_mut() {
        text.sections[0].style.color = text_color.colors[text_color.index];
        if text_color.timer.tick(time.delta()).just_finished() {
            text_color.index = (text_color.index + 1) % text_color.colors.len();
        }
    }
}
