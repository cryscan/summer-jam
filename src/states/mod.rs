use bevy::prelude::*;

pub mod game;
pub mod score;
pub mod title;

pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(&mut self, group: &mut bevy::app::PluginGroupBuilder) {
        group
            .add(title::TitlePlugin)
            .add(game::GamePlugin)
            .add(score::ScorePlugin);
    }
}
