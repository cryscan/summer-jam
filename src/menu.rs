use crate::{
    constants::*,
    game::Score,
    utils::{cleanup_system, escape_system},
    AppState, AudioVolume, ColorText, HintText, MusicTrack, TimeScale,
};
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioApp, AudioChannel, AudioControl};

pub struct MenuPlugin;

#[derive(Resource)]
struct ButtonAudio;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonStyle>()
            .add_audio_channel::<ButtonAudio>()
            .add_system_set(
                SystemSet::new()
                    .label(ButtonSystems)
                    .with_system(button_system)
                    .with_system(button_action)
                    .with_system(value_system)
                    .with_system(value_action),
            )
            .add_system(button_audio.after(ButtonSystems))
            .add_system_set(
                SystemSet::on_enter(AppState::Menu)
                    .with_system(enter_menu)
                    .with_system(make_menu),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Menu).with_system(cleanup_system::<Cleanup>),
            )
            .add_system_set(SystemSet::on_enter(AppState::Settings).with_system(make_settings))
            .add_system_set(SystemSet::on_update(AppState::Settings).with_system(escape_system))
            .add_system_set(
                SystemSet::on_exit(AppState::Settings).with_system(cleanup_system::<Cleanup>),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::Score)
                    .with_system(enter_score)
                    .with_system(make_score),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Score).with_system(cleanup_system::<Cleanup>),
            );
    }
}

#[derive(Component)]
struct Cleanup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub struct ButtonSystems;

#[derive(Clone, Copy, Component)]
enum ButtonAction {
    Play,
    Tutorial,
    Settings,
    Back,
}

#[derive(Clone, Copy, Component)]
enum ValueAction {
    AudioVolume(f32),
    MusicVolume(f32),
}

#[derive(Resource)]
struct ButtonStyle {
    button: Style,
    icon: Style,
    text: TextStyle,
}

impl FromWorld for ButtonStyle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        ButtonStyle {
            button: Style {
                size: Size::new(Val::Px(200.0), Val::Px(30.0)),
                position: UiRect {
                    left: Val::Percent(10.0),
                    ..Default::default()
                },
                margin: UiRect {
                    top: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            icon: Style {
                size: Size::new(Val::Px(20.0), Val::Auto),
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.0),
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Auto,
                },
                ..Default::default()
            },
            text: TextStyle {
                font: asset_server.load(FONT_KARMATIC),
                font_size: 20.0,
                color: BUTTON_TEXT_NORMAL_COLOR,
            },
        }
    }
}

fn enter_menu(
    mut time_scale: ResMut<TimeScale>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    volume: Res<AudioVolume>,
    mut music_track: ResMut<MusicTrack>,
) {
    time_scale.reset();
    if music_track.0 != MENU_MUSIC {
        audio.stop();
        audio.set_playback_rate(1.0);
        audio.set_volume(volume.music.into());
        audio.play(asset_server.load(MENU_MUSIC)).looped();

        music_track.0 = MENU_MUSIC;
    }
}

fn make_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_style: Res<ButtonStyle>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    style: Style {
                        position: UiRect {
                            left: Val::Percent(10.0),
                            ..Default::default()
                        },
                        margin: UiRect {
                            bottom: Val::Percent(20.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text::from_section(
                        "Bounce Up!",
                        TextStyle {
                            font: asset_server.load(FONT_ARCADE),
                            font_size: 50.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ColorText::new(FLIP_TEXT_COLORS.into(), 30.0 / MENU_MUSIC_BPM),
            ));

            parent.spawn((
                TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Percent(10.0),
                            top: Val::Percent(40.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: asset_server.load(FONT_INVASION),
                            font_size: 15.0,
                            color: HEALTH_BAR_COLOR,
                        },
                    )
                    .with_alignment(TextAlignment {
                        horizontal: HorizontalAlign::Left,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                HintText::new(480.0 / MENU_MUSIC_BPM),
            ));

            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.button.clone(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..Default::default()
                    },
                    ButtonAction::Play,
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(RIGHT_ICON)),
                        ..Default::default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section("Play", button_style.text.clone()),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.button.clone(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..Default::default()
                    },
                    ButtonAction::Tutorial,
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(RETICLE_ICON)),
                        ..Default::default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section("Practice", button_style.text.clone()),
                        ..Default::default()
                    });
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.button.clone(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..Default::default()
                    },
                    ButtonAction::Settings,
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(WRENCH_ICON)),
                        ..Default::default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section("Settings", button_style.text.clone()),
                        ..Default::default()
                    });
                });
        });
}

fn make_settings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_style: Res<ButtonStyle>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                style: Style {
                    position: UiRect {
                        left: Val::Percent(10.0),
                        ..Default::default()
                    },
                    margin: UiRect {
                        bottom: Val::Percent(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::from_section(
                    "Settings",
                    TextStyle {
                        font: asset_server.load(FONT_KARMATIC),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                )
                .with_alignment(TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                }),
                ..Default::default()
            });

            // audio volume
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(40.0)),
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        style: Style {
                            position: UiRect {
                                left: Val::Percent(10.0),
                                ..Default::default()
                            },
                            margin: UiRect {
                                right: Val::Percent(10.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: Text::from_section(
                            "Audio",
                            TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        )
                        .with_alignment(TextAlignment {
                            horizontal: HorizontalAlign::Center,
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                    for volume_setting in 0..=10 {
                        parent.spawn((
                            ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                                    margin: UiRect {
                                        left: Val::Px(2.0),
                                        right: Val::Px(2.0),
                                        ..Default::default()
                                    },
                                    ..button_style.button.clone()
                                },
                                background_color: SETTING_NORMAL_COLOR.into(),
                                ..Default::default()
                            },
                            ValueAction::AudioVolume(volume_setting as f32 / 10.0),
                        ));
                    }
                });

            // music volume
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(40.0)),
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        style: Style {
                            position: UiRect {
                                left: Val::Percent(10.0),
                                ..Default::default()
                            },
                            margin: UiRect {
                                right: Val::Percent(10.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: Text::from_section(
                            "Music",
                            TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        )
                        .with_alignment(TextAlignment {
                            horizontal: HorizontalAlign::Center,
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                    for volume_setting in 0..=10 {
                        parent.spawn((
                            ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                                    margin: UiRect {
                                        left: Val::Px(2.0),
                                        right: Val::Px(2.0),
                                        ..Default::default()
                                    },
                                    ..button_style.button.clone()
                                },
                                background_color: SETTING_NORMAL_COLOR.into(),
                                ..Default::default()
                            },
                            ValueAction::MusicVolume(volume_setting as f32 / 10.0),
                        ));
                    }
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.button.clone(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..Default::default()
                    },
                    ButtonAction::Back,
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(EXIT_ICON)),
                        ..Default::default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section("Back", button_style.text.clone()),
                        ..Default::default()
                    });
                });
        });
}

fn enter_score(mut time_scale: ResMut<TimeScale>) {
    time_scale.reset();
}

fn make_score(
    mut commands: Commands,
    time: Res<Time>,
    score: Res<Score>,
    asset_server: Res<AssetServer>,
    button_style: Res<ButtonStyle>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                background_color: Color::NONE.into(),
                ..Default::default()
            },
            Cleanup,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    style: Style {
                        position: UiRect {
                            left: Val::Percent(10.0),
                            ..Default::default()
                        },
                        margin: UiRect {
                            bottom: Val::Percent(20.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text::from_section(
                        "You Win!",
                        TextStyle {
                            font: asset_server.load(FONT_ARCADE),
                            font_size: 50.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ColorText::new(FLIP_TEXT_COLORS.into(), 30.0 / MENU_MUSIC_BPM),
            ));

            let term_style = Style {
                size: Size::new(Val::Percent(100.0), Val::Px(30.0)),
                position: UiRect {
                    left: Val::Percent(10.0),
                    ..Default::default()
                },
                margin: UiRect {
                    top: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    ..Default::default()
                },
                ..Default::default()
            };

            // time
            let time_passed = time.elapsed_seconds() - score.timestamp;
            parent.spawn(TextBundle {
                style: term_style.clone(),
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "Time: ".into(),
                            style: TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        },
                        TextSection {
                            value: format!("{time_passed:.2}"),
                            style: TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::GOLD,
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            });

            // player hits
            parent.spawn(TextBundle {
                style: term_style.clone(),
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "Hits: ".into(),
                            style: TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        },
                        TextSection {
                            value: score.hits.to_string(),
                            style: TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::GOLD,
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            });

            // player miss
            parent.spawn(TextBundle {
                style: term_style,
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "Miss: ".into(),
                            style: TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        },
                        TextSection {
                            value: score.miss.to_string(),
                            style: TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::GOLD,
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            });

            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.button.clone(),
                        background_color: BUTTON_NORMAL_COLOR.into(),
                        ..Default::default()
                    },
                    ButtonAction::Back,
                ))
                .with_children(|parent| {
                    parent.spawn(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(EXIT_ICON)),
                        ..Default::default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section("Back", button_style.text.clone()),
                        ..Default::default()
                    });
                });
        });
}

#[allow(clippy::type_complexity)]
fn button_audio(
    interaction_query: Query<
        (&Interaction, Option<&ButtonAction>),
        (Changed<Interaction>, With<Button>),
    >,
    audio: Res<AudioChannel<ButtonAudio>>,
    volume: Res<AudioVolume>,
    asset_server: Res<AssetServer>,
) {
    // let channel = AudioChannel::new("button".into());
    for (interaction, maybe_action) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                let volume = volume.effects * 0.5;
                audio.set_volume(volume.into());
                audio.play(asset_server.load(BUTTON_CLICK_AUDIO));
            }
            Interaction::Hovered => {
                if maybe_action.is_some() {
                    let volume = volume.effects * 0.5;
                    audio.set_volume(volume.into());
                    audio.play(asset_server.load(BUTTON_HOVER_AUDIO));
                }
            }
            Interaction::None => {}
        }
    }
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>, With<ButtonAction>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, children) in interaction_query.iter_mut() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(*child) {
                let text_color = &mut text.sections[0].style.color;
                match *interaction {
                    Interaction::Clicked => {
                        *text_color = BUTTON_TEXT_PRESSED_COLOR;
                        *color = BUTTON_PRESSED_COLOR.into();
                    }
                    Interaction::Hovered => {
                        *text_color = BUTTON_TEXT_HOVERED_COLOR;
                        *color = BUTTON_HOVERED_COLOR.into();
                    }
                    Interaction::None => {
                        *text_color = BUTTON_TEXT_NORMAL_COLOR;
                        *color = BUTTON_NORMAL_COLOR.into();
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn button_action(
    interaction_query: Query<(&Interaction, &ButtonAction), (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<State<AppState>>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            let state = match action {
                ButtonAction::Play => AppState::Battle,
                ButtonAction::Tutorial => AppState::Practice,
                ButtonAction::Settings => AppState::Settings,
                ButtonAction::Back => AppState::Menu,
            };
            app_state.set(state).unwrap();
        }
    }
}

#[allow(clippy::type_complexity)]
fn value_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &ValueAction), With<Button>>,
    volume: Res<AudioVolume>,
) {
    for (interaction, mut color, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => *color = SETTING_HOVERED_COLOR.into(),
            _ => {
                *color = SETTING_NORMAL_COLOR.into();
                match action {
                    ValueAction::AudioVolume(v) => {
                        if volume.effects >= *v {
                            *color = SETTING_ACTIVE_COLOR.into();
                        }
                    }
                    ValueAction::MusicVolume(v) => {
                        if volume.music >= *v {
                            *color = SETTING_ACTIVE_COLOR.into();
                        }
                    }
                };
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn value_action(
    interaction_query: Query<(&Interaction, &ValueAction), (Changed<Interaction>, With<Button>)>,
    mut volume: ResMut<AudioVolume>,
    audio: Res<Audio>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match action {
                ValueAction::AudioVolume(v) => volume.effects = *v,
                ValueAction::MusicVolume(v) => {
                    volume.music = *v;
                    audio.set_volume(volume.music.into());
                }
            }
        }
    }
}
