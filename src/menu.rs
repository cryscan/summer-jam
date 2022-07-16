use crate::{config::*, utils::cleanup_system, AppState, AudioVolume, TimeScale};
use bevy::prelude::*;
use bevy_kira_audio::Audio;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Cleanup>()
            .register_type::<TextColor>()
            .insert_resource(TextColorTimer(Timer::from_seconds(0.3, true)))
            .add_system_set(
                SystemSet::on_enter(AppState::Menu)
                    .with_system(enter_menu)
                    .with_system(make_menu),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Menu)
                    .with_system(update_menu)
                    .with_system(text_color)
                    .with_system(button_system),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Menu).with_system(cleanup_system::<Cleanup>),
            );
    }
}

const TITLE_COLORS: [Color; 2] = [Color::WHITE, Color::GOLD];

const NORMAL_BUTTON: Color = Color::NONE;
const HOVERED_BUTTON: Color = Color::WHITE;
const PRESSED_BUTTON: Color = Color::WHITE;

const NORMAL_BUTTON_TEXT: Color = Color::WHITE;
const HOVERED_BUTTON_TEXT: Color = Color::BLACK;
const PRESSED_BUTTON_TEXT: Color = Color::BLACK;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct Cleanup;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct TextColor {
    pub colors: Vec<Color>,
    pub index: usize,
}

#[derive(Deref, DerefMut)]
struct TextColorTimer(Timer);

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

            let button_style = Style {
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
            };
            let button_icon_style = Style {
                size: Size::new(Val::Px(20.0), Val::Auto),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.0),
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Auto,
                },
                ..Default::default()
            };
            let button_text_style = TextStyle {
                font: asset_server.load(FONT_KARMATIC),
                font_size: 20.0,
                color: NORMAL_BUTTON_TEXT,
            };

            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: button_icon_style.clone(),
                        image: UiImage(asset_server.load(RIGHT_ICON)),
                        ..Default::default()
                    });
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Play",
                            button_text_style.clone(),
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style.clone(),
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: button_icon_style.clone(),
                        image: UiImage(asset_server.load(RIGHT_ICON)),
                        ..Default::default()
                    });
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Tutorial",
                            button_text_style.clone(),
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: button_style,
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style: button_icon_style,
                        image: UiImage(asset_server.load(WRENCH_ICON)),
                        ..Default::default()
                    });
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Settings",
                            button_text_style.clone(),
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
        });
}

fn update_menu(mut app_state: ResMut<State<AppState>>, mut input: ResMut<Input<MouseButton>>) {
    if input.just_pressed(MouseButton::Left) {
        input.reset(MouseButton::Left);
        app_state.set(AppState::Game).unwrap();
    }
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
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
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
