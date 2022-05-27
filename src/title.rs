use crate::{
    config::*,
    utils::{cleanup_system, TimeScale},
    AppState,
};
use bevy::prelude::*;

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColorTimer(Timer::from_seconds(0.2, true)))
            .add_system_set(
                SystemSet::on_enter(AppState::Title)
                    .with_system(enter_title)
                    .with_system(make_title),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Title)
                    .with_system(update_title)
                    .with_system(title_color),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Title).with_system(cleanup_system::<Cleanup>),
            );
    }
}

#[derive(Component)]
struct Cleanup;

#[derive(Default, Clone, Copy, Component)]
struct ColorText {
    bright: Color,
    dark: Color,
}

struct ColorTimer(Timer);

fn enter_title(mut time_scale: ResMut<TimeScale>) {
    time_scale.reset();
}

fn make_title(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Entering Title");

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

fn update_title(mut app_state: ResMut<State<AppState>>, mut input: ResMut<Input<MouseButton>>) {
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
    if timer.0.tick(time.delta()).just_finished() {
        for (mut text, color) in query.iter_mut() {
            text.sections[0].style.color = match *color_flag {
                true => color.bright,
                false => color.dark,
            };
        }

        *color_flag = !*color_flag;
    }
}
