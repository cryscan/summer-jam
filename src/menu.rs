use crate::{config::*, utils::cleanup_system, AppState, AudioVolume, TimeScale};
use bevy::prelude::*;
use bevy_kira_audio::Audio;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Cleanup>()
            .register_type::<ColorText>()
            .insert_resource(ColorTimer(Timer::from_seconds(0.2, true)))
            .add_system_set(
                SystemSet::on_enter(AppState::Menu)
                    .with_system(enter_menu)
                    .with_system(make_menu),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Menu)
                    .with_system(update_menu)
                    .with_system(title_color),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Menu).with_system(cleanup_system::<Cleanup>),
            );
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct Cleanup;

#[derive(Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
struct ColorText {
    bright: Color,
    dark: Color,
}

#[derive(Deref, DerefMut)]
struct ColorTimer(Timer);

fn enter_menu(
    mut time_scale: ResMut<TimeScale>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    volume: Res<AudioVolume>,
) {
    time_scale.reset();

    audio.stop();
    audio.set_playback_rate(1.0);
    audio.set_volume(volume.music);
    audio.play_looped(asset_server.load(TITLE_MUSIC));
}

fn make_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Entering Menu");

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(Cleanup)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                style: Style {
                    position: Rect {
                        left: Val::Percent(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::with_section(
                    "Bounce Up!",
                    TextStyle {
                        font: asset_server.load(FONT_ARCADE),
                        font_size: 50.0,
                        color: Color::WHITE,
                    },
                    TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });

            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        position: Rect {
                            left: Val::Percent(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "Click to Play",
                        TextStyle {
                            font: asset_server.load(FONT_ARCADE),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                })
                .insert(ColorText {
                    bright: Color::GOLD,
                    dark: Color::GRAY,
                });
        });
}

fn update_menu(mut app_state: ResMut<State<AppState>>, mut input: ResMut<Input<MouseButton>>) {
    if input.just_pressed(MouseButton::Left) {
        input.reset(MouseButton::Left);
        app_state.set(AppState::Game).unwrap();
    }
}

fn title_color(
    time: Res<Time>,
    mut timer: ResMut<ColorTimer>,
    mut color_flag: Local<bool>,
    mut query: Query<(&mut Text, &ColorText)>,
) {
    if timer.tick(time.delta()).just_finished() {
        for (mut text, color) in query.iter_mut() {
            text.sections[0].style.color = match *color_flag {
                true => color.bright,
                false => color.dark,
            };
        }

        *color_flag = !*color_flag;
    }
}
