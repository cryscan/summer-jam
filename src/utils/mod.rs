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

pub struct TimeScale(pub f32);

impl Default for TimeScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl TimeScale {
    pub fn reset(&mut self) {
        self.0 = 1.0;
    }
}

pub struct MusicVolume(pub f32);
