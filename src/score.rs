use crate::{constants::*, AppState, ColorText, TimeScale};
use bevy::prelude::*;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>().add_system_set(
            SystemSet::on_enter(AppState::Win)
                .with_system(enter_score)
                .with_system(make_ui),
        );
    }
}

#[derive(Resource)]
pub struct Score {
    pub timestamp: f32,
    pub hits: i32,
    pub miss: i32,
}

impl FromWorld for Score {
    fn from_world(world: &mut World) -> Self {
        let time = world.resource::<Time>();
        Self {
            timestamp: time.elapsed_seconds(),
            hits: 0,
            miss: 0,
        }
    }
}

fn enter_score(mut time_scale: ResMut<TimeScale>) {
    time_scale.reset();
}

fn make_ui(
    mut commands: Commands,
    time: Res<Time>,
    score: Res<Score>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
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
        });
}
