use bevy::{input::system::exit_on_esc_system, prelude::*};
use wasm_bindgen::prelude::*;

mod main_menu;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InGame,
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(WindowDescriptor {
            title: "Cleanup!".into(),
            width: 480.0,
            height: 640.0,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_state(AppState::MainMenu)
        .add_startup_system(setup.system())
        .add_system(lock_release_cursor.system())
        .add_system(exit_on_esc_system.system())
        .add_plugin(main_menu::MainMenuPlugin);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    /* commands.spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
        transform: Transform::from_xyz(0.0, -0.0, 0.0),
        sprite: Sprite::new(Vec2::new(32.0, 32.0)),
        ..Default::default()
    }); */
}

fn lock_release_cursor(app_state: Res<State<AppState>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        match app_state.current() {
            AppState::InGame => {
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
