mod collide;
mod damp;
mod interpolation;

use crate::AppState;
use bevy::prelude::*;

pub use collide::*;
pub use damp::*;
pub use interpolation::*;

pub fn cleanup_system<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn escape_system(mut app_state: ResMut<State<AppState>>, mut input: ResMut<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        input.reset(KeyCode::Escape);
        app_state.set(AppState::Menu).unwrap();
    }
}
