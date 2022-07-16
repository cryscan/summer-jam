use crate::{
    config::*,
    utils::{cleanup_system, escape_system},
    AppState, AudioVolume, MusicTrack, TimeScale,
};
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Cleanup>()
            .register_type::<TextColor>()
            .register_type::<MenuButtonAction>()
            .register_type::<SettingButtonAction>()
            .insert_resource(TextColorTimer(Timer::from_seconds(0.3, true)))
            .init_resource::<ButtonStyle>()
            .add_system(text_color)
            .add_system_set(
                SystemSet::new()
                    .label(ButtonSystems)
                    .with_system(menu_button_system)
                    .with_system(menu_action)
                    .with_system(setting_button_system)
                    .with_system(setting_action),
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
            );
    }
}

const TITLE_COLORS: [Color; 2] = [Color::WHITE, Color::GOLD];

const NORMAL_BUTTON: Color = Color::NONE;
const HOVERED_BUTTON: Color = Color::WHITE;
const PRESSED_BUTTON: Color = Color::WHITE;

const NORMAL_SETTING_BUTTON: Color = Color::BLACK;
const ACTIVE_SETTING_BUTTON: Color = Color::WHITE;
const HOVERED_SETTING_BUTTON: Color = Color::GRAY;

const NORMAL_BUTTON_TEXT: Color = Color::WHITE;
const HOVERED_BUTTON_TEXT: Color = Color::BLACK;
const PRESSED_BUTTON_TEXT: Color = Color::BLACK;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct Cleanup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub struct ButtonSystems;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct TextColor {
    pub colors: Vec<Color>,
    pub index: usize,
}

#[derive(Clone, Copy, Component, Reflect)]
#[reflect(Component)]
enum SettingButtonAction {
    AudioVolume(f32),
    MusicVolume(f32),
}

impl Default for SettingButtonAction {
    fn default() -> Self {
        Self::MusicVolume(0.0)
    }
}

#[derive(Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
enum MenuButtonAction {
    #[default]
    Play,
    Tutorial,
    Settings,
    BackToMenu,
}

#[derive(Deref, DerefMut)]
struct TextColorTimer(Timer);

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
                position: Rect {
                    left: Val::Percent(10.0),
                    ..Default::default()
                },
                margin: Rect {
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
                position: Rect {
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
                color: NORMAL_BUTTON_TEXT,
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
    info!("Entering Menu");

    time_scale.reset();
    if music_track.0 != MENU_MUSIC {
        audio.stop();
        audio.set_playback_rate(1.0);
        audio.set_volume(volume.music);
        audio.play_looped(asset_server.load(MENU_MUSIC));

        music_track.0 = MENU_MUSIC;
    }
}

fn make_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_style: Res<ButtonStyle>,
) {
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
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        position: Rect {
                            left: Val::Percent(10.0),
                            ..Default::default()
                        },
                        margin: Rect {
                            bottom: Val::Percent(20.0),
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
                })
                .insert(TextColor {
                    colors: TITLE_COLORS.into(),
                    ..Default::default()
                });

            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.button.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .insert(MenuButtonAction::Play)
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(RIGHT_ICON)),
                        ..Default::default()
                    });
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Play",
                            button_style.text.clone(),
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.button.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .insert(MenuButtonAction::Tutorial)
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(RIGHT_ICON)),
                        ..Default::default()
                    });
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Tutorial",
                            button_style.text.clone(),
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.button.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .insert(MenuButtonAction::Settings)
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(WRENCH_ICON)),
                        ..Default::default()
                    });
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Settings",
                            button_style.text.clone(),
                            Default::default(),
                        ),
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
    info!("Entering Settings");

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
                    margin: Rect {
                        bottom: Val::Percent(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::with_section(
                    "Settings",
                    TextStyle {
                        font: asset_server.load(FONT_KARMATIC),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                    TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });

            // audio volume
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(40.0)),
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        style: Style {
                            position: Rect {
                                left: Val::Percent(10.0),
                                ..Default::default()
                            },
                            margin: Rect {
                                right: Val::Percent(10.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: Text::with_section(
                            "Audio",
                            TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    });
                    for volume_setting in 0..=10 {
                        parent
                            .spawn_bundle(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                                    margin: Rect {
                                        left: Val::Px(2.0),
                                        right: Val::Px(2.0),
                                        ..Default::default()
                                    },
                                    ..button_style.button.clone()
                                },
                                color: NORMAL_SETTING_BUTTON.into(),
                                ..Default::default()
                            })
                            .insert(SettingButtonAction::AudioVolume(
                                volume_setting as f32 / 10.0,
                            ));
                    }
                });

            // music volume
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(40.0)),
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        style: Style {
                            position: Rect {
                                left: Val::Percent(10.0),
                                ..Default::default()
                            },
                            margin: Rect {
                                right: Val::Percent(10.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: Text::with_section(
                            "Music",
                            TextStyle {
                                font: asset_server.load(FONT_KARMATIC),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                            TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    });
                    for volume_setting in 0..=10 {
                        parent
                            .spawn_bundle(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                                    margin: Rect {
                                        left: Val::Px(2.0),
                                        right: Val::Px(2.0),
                                        ..Default::default()
                                    },
                                    ..button_style.button.clone()
                                },
                                color: NORMAL_SETTING_BUTTON.into(),
                                ..Default::default()
                            })
                            .insert(SettingButtonAction::MusicVolume(
                                volume_setting as f32 / 10.0,
                            ));
                    }
                });

            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.button.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .insert(MenuButtonAction::BackToMenu)
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: button_style.icon.clone(),
                        image: UiImage(asset_server.load(EXIT_ICON)),
                        ..Default::default()
                    });
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Back",
                            button_style.text.clone(),
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
        });
}

fn text_color(
    time: Res<Time>,
    mut timer: ResMut<TextColorTimer>,
    mut query: Query<(&mut Text, &mut TextColor)>,
) {
    timer.tick(time.delta());
    for (mut text, mut text_color) in query.iter_mut() {
        text.sections[0].style.color = text_color.colors[text_color.index];
        if timer.just_finished() {
            text_color.index = (text_color.index + 1) % text_color.colors.len();
        }
    }
}

#[allow(clippy::type_complexity)]
fn button_audio(
    interaction_query: Query<
        (&Interaction, Option<&MenuButtonAction>),
        (Changed<Interaction>, With<Button>),
    >,
    audio: Res<Audio>,
    volume: Res<AudioVolume>,
    asset_server: Res<AssetServer>,
) {
    let channel = AudioChannel::new("button".into());
    for (interaction, maybe_action) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                let volume = volume.effects * 0.5;
                audio.set_volume_in_channel(volume, &channel);
                audio.play_in_channel(asset_server.load(BUTTON_CLICK_AUDIO), &channel);
            }
            Interaction::Hovered => {
                if maybe_action.is_some() {
                    let volume = volume.effects * 0.5;
                    audio.set_volume_in_channel(volume, &channel);
                    audio.play_in_channel(asset_server.load(BUTTON_HOVER_AUDIO), &channel);
                }
            }
            Interaction::None => {}
        }
    }
}

#[allow(clippy::type_complexity)]
fn menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>, With<MenuButtonAction>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, children) in interaction_query.iter_mut() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(*child) {
                let text_color = &mut text.sections[0].style.color;
                match *interaction {
                    Interaction::Clicked => {
                        *text_color = PRESSED_BUTTON_TEXT;
                        *color = PRESSED_BUTTON.into();
                    }
                    Interaction::Hovered => {
                        *text_color = HOVERED_BUTTON_TEXT;
                        *color = HOVERED_BUTTON.into();
                    }
                    Interaction::None => {
                        *text_color = NORMAL_BUTTON_TEXT;
                        *color = NORMAL_BUTTON.into();
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<State<AppState>>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            let state = match action {
                MenuButtonAction::Play => AppState::Game,
                MenuButtonAction::Tutorial => AppState::Tutorial,
                MenuButtonAction::Settings => AppState::Settings,
                MenuButtonAction::BackToMenu => AppState::Menu,
            };
            app_state.set(state).unwrap();
        }
    }
}

#[allow(clippy::type_complexity)]
fn setting_button_system(
    mut interaction_query: Query<(&Interaction, &mut UiColor, &SettingButtonAction), With<Button>>,
    volume: Res<AudioVolume>,
) {
    for (interaction, mut color, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => *color = HOVERED_SETTING_BUTTON.into(),
            _ => {
                *color = NORMAL_SETTING_BUTTON.into();
                match action {
                    SettingButtonAction::AudioVolume(v) => {
                        if volume.effects >= *v {
                            *color = ACTIVE_SETTING_BUTTON.into();
                        }
                    }
                    SettingButtonAction::MusicVolume(v) => {
                        if volume.music >= *v {
                            *color = ACTIVE_SETTING_BUTTON.into();
                        }
                    }
                };
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn setting_action(
    interaction_query: Query<
        (&Interaction, &SettingButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut volume: ResMut<AudioVolume>,
    audio: Res<Audio>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match action {
                SettingButtonAction::AudioVolume(v) => volume.effects = *v,
                SettingButtonAction::MusicVolume(v) => {
                    volume.music = *v;
                    audio.set_volume(volume.music)
                }
            }
        }
    }
}
