use crate::{config::*, utils::Damp, TimeScale};
use bevy::{math::Vec3Swizzles, prelude::*};

pub struct Slits {
    pub count: usize,
    pub index: usize,
}

impl Default for Slits {
    fn default() -> Self {
        Self {
            count: (ARENA_WIDTH / SLIT_BLOCK_WIDTH) as usize - 1,
            index: 0,
        }
    }
}

#[derive(Component)]
pub struct SlitBlock {
    pub index: usize,
}

pub fn slits_system(
    time: Res<Time>,
    time_scale: Res<TimeScale>,
    slits: Res<Slits>,
    mut query: Query<(&mut Transform, &SlitBlock)>,
) {
    for (mut transform, slit) in query.iter_mut() {
        let offset = if slit.index < slits.index { 0 } else { 1 };
        let left = (slit.index + offset) as f32 * SLIT_BLOCK_WIDTH;
        let position = Vec2::new(
            left + (SLIT_BLOCK_WIDTH - ARENA_WIDTH) / 2.0,
            SLIT_POSITION_VERTICAL,
        );
        let position = transform.translation.xy().damp(
            position,
            SLIT_BLOCK_DAMP,
            time.delta_seconds() * time_scale.0,
        );
        transform.translation = position.extend(0.1);
    }
}
