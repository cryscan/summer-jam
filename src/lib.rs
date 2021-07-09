use bevy::prelude::*;
use wasm_bindgen::prelude::*;

mod config;
mod game;
mod states;
mod utils;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Title,
    Game,
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(WindowDescriptor {
            title: "Cleanup!".into(),
            width: config::ARENA_WIDTH,
            height: config::ARENA_HEIGHT,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_state(AppState::Title)
        .add_startup_system(setup.system())
        .add_system(lock_release_cursor.system())
        .add_plugin(states::TitlePlugin)
        .add_plugin(states::GamePlugin);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
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
