use bevy::prelude::*;

#[derive(Component)]
pub struct Hint(pub Entity);

pub fn hint_system(
    query: Query<(&Transform, &Hint)>,
    mut hint_query: Query<&mut Transform, Without<Hint>>,
) {
    for (transform, hint) in query.iter() {
        let x = transform.translation.x;
        if let Ok(mut transform) = hint_query.get_mut(hint.0) {
            transform.translation.x = x;
        }
    }
}
