use crate::AppState;
use bevy::prelude::*;

struct TitleText;

struct ColorText;
struct ColorTimer(Timer);

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(50.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "Cleanup!",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
                Default::default(),
            ),
            ..Default::default()
        })
        .insert(TitleText);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(40.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "Press Enter to Play",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
                Default::default(),
            ),
            ..Default::default()
        })
        .insert(TitleText)
        .insert(ColorText);
}

fn main_menu_system(
    mut commands: Commands,
    mut app_state: ResMut<State<AppState>>,
    input: Res<Input<KeyCode>>,
    query: Query<Entity, With<TitleText>>,
) {
    if input.pressed(KeyCode::Return) {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        app_state.set(AppState::InGame).unwrap();
    }
}

fn title_color_system(
    time: Res<Time>,
    mut timer: ResMut<ColorTimer>,
    mut color_flag: Local<bool>,
    mut query: Query<&mut Text, With<ColorText>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut text in query.iter_mut() {
            if *color_flag {
                text.sections[0].style.color = Color::GOLD;
            } else {
                text.sections[0].style.color = Color::GRAY;
            }
        }

        *color_flag = !*color_flag;
    }
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ColorTimer(Timer::from_seconds(0.2, true)))
            .add_system_set(
                SystemSet::on_enter(AppState::MainMenu).with_system(setup_main_menu.system()),
            )
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu)
                    .with_system(main_menu_system.system())
                    .with_system(title_color_system.system()),
            );
    }
}
