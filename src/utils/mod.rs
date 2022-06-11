mod collide;
mod damp;
mod interpolation;

use bevy::prelude::*;

pub use collide::*;
pub use damp::*;
pub use interpolation::*;

pub fn cleanup_system<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
