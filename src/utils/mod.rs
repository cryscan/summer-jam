mod collide;
mod damp;
mod interpolation;

use bevy::{ecs::component::Component, prelude::*};

pub use collide::{collide, Collider, Hit, Intersection, Penetration};
pub use damp::Damp;
pub use interpolation::Interpolation;

pub fn cleanup_system<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
