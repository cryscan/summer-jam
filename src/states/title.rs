use crate::{config::*, AppState};
use bevy::prelude::*;

struct TitleStateTag;

#[derive(Default, Clone, Copy)]
struct ColorText {
    bright: Color,
    dark: Color,
}

struct ColorTimer(Timer);

fn make_title(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Entering Title");

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .insert(TitleStateTag)
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
                    "Cleanup!",
                    TextStyle {
                        font: asset_server.load(FONT_FIRA_SANS),
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
                            font: asset_server.load(FONT_FIRA_MONO),
                            font_size: 20.0,
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

fn cleanup_title(mut commands: Commands, query: Query<Entity, With<TitleStateTag>>) {
    println!("Cleaning-up Title");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ColorTimer(Timer::from_seconds(0.2, true)))
            .add_system_set(SystemSet::on_enter(AppState::Title).with_system(make_title))
            .add_system_set(
                SystemSet::on_update(AppState::Title)
                    .with_system(update_title)
                    .with_system(title_color),
            )
            .add_system_set(SystemSet::on_exit(AppState::Title).with_system(cleanup_title));
    }
}
