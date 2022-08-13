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
pub struct ColorText {
    timer: Timer,
    colors: Vec<Color>,
    index: usize,
}

impl ColorText {
    pub fn new(colors: Vec<Color>, duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, true),
            colors,
            index: 0,
        }
    }
}

#[derive(Component)]
pub struct HintText {
    index: usize,
    timer: Timer,
}

impl HintText {
    const HINT_TEXTS: [&'static str; 3] = [
        "Control your ball speed!",
        "Can your paddle catch the ball on its own?",
        "Try to bounce; not to push!",
    ];

    pub fn new(duration: f32) -> Self {
        Self {
            index: 0,
            timer: Timer::from_seconds(duration, true),
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
        .add_system(color_text_system)
        .add_system(hint_text_system)
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
    commands.spawn_bundle(Camera2dBundle::default()).insert(
        UiCameraConfig {
            ..default()
        }
    );
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

fn color_text_system(time: Res<Time>, mut query: Query<(&mut Text, &mut ColorText)>) {
    for (mut text, mut color_text) in query.iter_mut() {
        text.sections[0].style.color = color_text.colors[color_text.index];
        if color_text.timer.tick(time.delta()).just_finished() {
            color_text.index = (color_text.index + 1) % color_text.colors.len();
        }
    }
}

pub fn hint_text_system(time: Res<Time>, mut query: Query<(&mut Text, &mut HintText)>) {
    for (mut text, mut hint) in query.iter_mut() {
        text.sections[0].value = HintText::HINT_TEXTS[hint.index].into();

        if hint.timer.tick(time.delta()).just_finished() {
            let len = HintText::HINT_TEXTS.len();
            let mut next = fastrand::usize(0..len);
            if next == hint.index {
                next = (hint.index + 1) % len;
            }
            hint.index = next;
        }
    }
}
